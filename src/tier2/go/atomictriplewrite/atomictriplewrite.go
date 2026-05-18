// k1s0 tier2 atomic 三表書込 Go 実装
// Rust 実装（atomic_triple_write.rs）と semantic 等価な Go 版
// State change / Outbox / Audit event を同一 DB トランザクションで書く
// TLA+ の P1-P4 invariant と double-bind する（src/formal/dafny/AtomicThreeTableWrite.dfy）
//
// P1: aggregate 状態変更時、必ず Outbox + Audit が同一 txn に書込
// P2: Outbox 投入失敗時、aggregate 状態変更も rollback
// P3: tenant_id が GUC と aggregate 行で一致しない場合、操作を reject
// P4: pii_segregated の全アクセスを audit_event に記録

// パッケージ名: atomictriplewrite
package atomictriplewrite

import (
	// context パッケージ: Execute の ctx 引数に使用する
	"context"
	// database/sql パッケージ: *sql.Tx による実 transaction に使用する
	"database/sql"
	// errors パッケージ: エラー生成に使用する
	"errors"
	// fmt パッケージ: SQL 文字列フォーマットに使用する
	"fmt"
	// strings パッケージ: SQL エスケープに使用する
	"strings"
	// time パッケージ: committed_at の記録に使用する
	"time"

	// uuid パッケージ: aggregate_id / outbox_id / audit_event_id に使用する
	"github.com/google/uuid"
	// tenantcontext パッケージ: TenantContext を受け取る
	"github.com/k1s0/tier2/tenantcontext"
	// pgx stdlib ドライバ: database/sql 互換ドライバとして pgx を登録する
	_ "github.com/jackc/pgx/v5/stdlib"
)

// TableClass: 書込対象テーブルクラス（10_テナント分離適合仕様.md の 4 class と一致する）
type TableClass int

const (
	// TableClassTenantScoped: tenant_id 必須、RLS FORCE
	TableClassTenantScoped TableClass = iota
	// TableClassTenantMaster: tenant_id 必須、role 制限付き RLS FORCE
	TableClassTenantMaster
	// TableClassPlatformGlobal: tenant_id 無し、RLS 無効
	TableClassPlatformGlobal
	// TableClassPiiSegregated: tenant_id 必須 + purpose check + pgaudit 全アクセス
	TableClassPiiSegregated
)

// String: TableClass を SQL コメント用文字列に変換する
func (tc TableClass) String() string {
	// 各クラスを文字列にマッピングする
	switch tc {
	case TableClassTenantScoped:
		// tenant_scoped クラスの文字列表現
		return "TenantScoped"
	case TableClassTenantMaster:
		// tenant_master クラスの文字列表現
		return "TenantMaster"
	case TableClassPlatformGlobal:
		// platform_global クラスの文字列表現
		return "PlatformGlobal"
	case TableClassPiiSegregated:
		// pii_segregated クラスの文字列表現
		return "PiiSegregated"
	default:
		// 未定義クラスは Unknown を返す
		return "Unknown"
	}
}

// AtomicWriteError: atomic 三表書込のエラー型
type AtomicWriteError struct {
	// エラー種別を識別するコード
	Code string
	// エラーメッセージ
	Message string
}

// Error: error インターフェースの実装
func (e *AtomicWriteError) Error() string {
	// コードとメッセージを結合して返す
	return fmt.Sprintf("[%s] %s", e.Code, e.Message)
}

var (
	// ErrOutboxInsertFailed: P2 — Outbox 投入失敗（rollback が必要）
	ErrOutboxInsertFailed = errors.New("outbox insert failed, transaction rolled back")
	// ErrTenantIdMismatch: P3 — tenant_id 不一致（GUC と aggregate 行の tenant_id が一致しない）
	ErrTenantIdMismatch = errors.New("tenant_id mismatch: guc and aggregate row differ")
	// ErrPiiAuditFailed: P4 — pii_segregated の audit_event 記録失敗
	ErrPiiAuditFailed = errors.New("pii audit record failed")
	// ErrTransactionFailed: 汎用 DB トランザクションエラー
	ErrTransactionFailed = errors.New("transaction error")
)

// StateChange: aggregate の状態変更を表す構造体（P1 の state_change に対応する）
type StateChange struct {
	// 変更対象の aggregate ID
	AggregateID uuid.UUID
	// 変更対象の tenant_id（TenantContext.TenantID() と一致している必要がある）
	TenantID uuid.UUID
	// テーブルクラス（どの class の table を書込むかを示す）
	TableClass TableClass
	// 変更内容のシリアライズ済みペイロード（JSON 文字列）
	Payload string
	// aggregate バージョン（楽観的ロックに使用する）
	Version int64
}

// TripleWriteResult: atomic 三表書込の結果
type TripleWriteResult struct {
	// 書込んだ aggregate ID
	AggregateID uuid.UUID
	// 書込んだ outbox エントリの ID
	OutboxID uuid.UUID
	// 書込んだ audit_event の ID
	AuditEventID uuid.UUID
	// 書込完了日時
	CommittedAt time.Time
}

// AtomicTripleWrite: atomic 三表書込の実行エンジン
// database/sql の *sql.Tx を受け取る Execute メソッドを持つ
// TenantContext を使って GUC 注入と tenant_id 検証を行う
type AtomicTripleWrite struct {
	// テナントコンテキスト（GUC 注入・tenant_id 検証に使用する）
	context *tenantcontext.TenantContext
}

// New: AtomicTripleWrite を生成する（TenantContext を受け取る）
func New(ctx *tenantcontext.TenantContext) *AtomicTripleWrite {
	// TenantContext を格納した AtomicTripleWrite を返す
	return &AtomicTripleWrite{context: ctx}
}

// VerifyTenantID: P3 — tenant_id 一致を検証する
// StateChange の TenantID が TenantContext の TenantID と一致しない場合はエラーを返す
func (a *AtomicTripleWrite) VerifyTenantID(change *StateChange) error {
	// GUC の tenant_id と aggregate の tenant_id を比較する
	gucTenantID := a.context.TenantID()
	if gucTenantID != change.TenantID {
		// P3 違反: tenant_id 不一致でエラーを返す
		return fmt.Errorf("%w: guc=%s, row=%s", ErrTenantIdMismatch, gucTenantID, change.TenantID)
	}
	// tenant_id が一致した場合は nil を返す
	return nil
}

// VerifyPiiAuditRequired: P4 — pii_segregated アクセスが audit_event 必須かを返す
func (a *AtomicTripleWrite) VerifyPiiAuditRequired(change *StateChange) bool {
	// PiiSegregated の場合は必ず audit_event を記録する（true を返す）
	return change.TableClass == TableClassPiiSegregated
}

// BuildTripleWriteSQL: P1 の atomic write に必要な SQL 文字列を生成する（デバッグ・テスト用）
// 実際の DB 実行は Execute() が *sql.Tx 経由で行う
func (a *AtomicTripleWrite) BuildTripleWriteSQL(change *StateChange) (string, error) {
	// P3: tenant_id 一致を事前検証する
	if err := a.VerifyTenantID(change); err != nil {
		// 検証失敗の場合は空文字とエラーを返す
		return "", err
	}
	// outbox エントリの ID を生成する
	outboxID := uuid.New()
	// audit_event の ID を生成する
	auditID := uuid.New()
	// 現在時刻を RFC3339 形式で取得する
	now := time.Now().UTC().Format(time.RFC3339Nano)
	// SET LOCAL GUC 注入 SQL を取得する（4 GUC 全て）
	setGUC := a.context.ToSetLocalSQL()
	// payload の single quote をエスケープする（SQL injection 対策）
	escapedPayload := strings.ReplaceAll(change.Payload, "'", "''")
	// P1: state_change + outbox + audit_event を BEGIN 〜 COMMIT の間に書く
	sql := fmt.Sprintf(`
BEGIN;
%s

-- P1: state_change (aggregate テーブルへの書込)
INSERT INTO k1s0.domain_event (id, aggregate_id, tenant_id, event_kind, payload, version, created_at)
VALUES ('%s', '%s', current_setting('app.tenant_id')::uuid, 'StateChange', '%s'::jsonb, %d, '%s');

-- P1: outbox_message (Debezium CDC 経由で Kafka に転送される)
INSERT INTO k1s0.outbox_message (id, aggregate_id, tenant_id, event_kind, payload, created_at)
VALUES ('%s', '%s', current_setting('app.tenant_id')::uuid, 'OutboxRelay', '%s'::jsonb, '%s');

-- P1 + P4: audit_event (全操作で記録、pii_segregated は pgaudit も併用)
INSERT INTO k1s0.audit_event (id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)
VALUES ('%s', '%s', current_setting('app.tenant_id')::uuid, current_setting('app.actor_id'), current_setting('app.purpose'), '%s', '%s'::jsonb, '%s');

COMMIT;
`,
		setGUC,
		auditID.String(), change.AggregateID.String(), escapedPayload, change.Version, now,
		outboxID.String(), change.AggregateID.String(), escapedPayload, now,
		auditID.String(), change.AggregateID.String(), change.TableClass.String(), escapedPayload, now,
	)
	// 生成した SQL 文字列を返す
	return sql, nil
}

// Execute: P1-P4 — atomic 三表書込を実行する非同期メソッド（実 *sql.Tx を使用する）
// tx: 呼び出し元が BEGIN した *sql.Tx を受け取る
// 呼び出し元は Ok 返却後に tx.Commit() を呼ぶ。エラー返却後は tx.Rollback() を呼ぶ。
// P1: state_change / outbox / audit_event の 3 INSERT を同一 tx で実行する
// P2: outbox INSERT が失敗した場合は ErrOutboxInsertFailed を返し、呼び出し元が Rollback する
// P3: tenant_id が GUC と一致しない場合は即座に reject して ErrTenantIdMismatch を返す
// P4: pii_segregated テーブルへのアクセスは audit_event に記録してから tx を実行する
func (a *AtomicTripleWrite) Execute(ctx context.Context, tx *sql.Tx, change *StateChange) (*TripleWriteResult, error) {
	// ctx の cancel を確認する（ctx.Done() で cancel 済みの場合は即座にエラーを返す）
	select {
	case <-ctx.Done():
		// context がキャンセルされた場合はエラーを返す
		return nil, fmt.Errorf("%w: context cancelled", ErrTransactionFailed)
	default:
		// context がアクティブな場合は処理を続行する
	}

	// P3: tenant_id 一致を事前検証する（GUC と aggregate 行の tenant_id が一致しない場合は即座にエラー）
	if err := a.VerifyTenantID(change); err != nil {
		// 検証失敗の場合はエラーを返す（呼び出し元が Rollback を呼ぶ）
		return nil, err
	}

	// P4: pii_segregated の場合は audit_event への記録が必須であることを確認する
	_ = a.VerifyPiiAuditRequired(change)

	// P3: SET LOCAL で 4 GUC を txn スコープに注入する（RLS FORCE が参照する）
	setGUC := a.context.ToSetLocalSQL()
	// SET LOCAL GUC を tx 内で実行する（transaction スコープのみ有効）
	if _, err := tx.ExecContext(ctx, setGUC); err != nil {
		// GUC 注入失敗はトランザクションエラーとして返す
		return nil, fmt.Errorf("%w: set local guc failed: %v", ErrTransactionFailed, err)
	}

	// outbox エントリの ID を生成する（P1 の atomic 三表書込で使用する）
	outboxID := uuid.New()
	// audit_event の ID を生成する（domain_event と audit_event で共有する）
	auditEventID := uuid.New()
	// 書込完了日時を記録する（3 INSERT で統一した timestamp を使用する）
	committedAt := time.Now().UTC()

	// P1: k1s0.domain_event テーブルに INSERT する（aggregate 状態変更の永続化）
	// current_setting('app.tenant_id')::uuid を使って RLS FORCE の tenant_id を注入する
	const domainEventSQL = `
		INSERT INTO k1s0.domain_event
			(id, aggregate_id, tenant_id, event_kind, payload, version, created_at)
		VALUES
			($1, $2, current_setting('app.tenant_id')::uuid, 'StateChange', $3::jsonb, $4, $5)
	`
	// domain_event INSERT を tx 内で実行する（P1 の atomic 書込を保証する）
	if _, err := tx.ExecContext(ctx, domainEventSQL,
		// audit_event_id を domain_event の主キーとして使用する
		auditEventID,
		// 変更対象の aggregate ID をバインドする
		change.AggregateID,
		// ペイロードを jsonb 文字列としてバインドする
		change.Payload,
		// aggregate バージョンをバインドする（楽観的ロックに使用する）
		change.Version,
		// 書込完了日時をバインドする
		committedAt,
	); err != nil {
		// domain_event INSERT 失敗はトランザクションエラーとして返す
		return nil, fmt.Errorf("%w: domain_event insert failed: %v", ErrTransactionFailed, err)
	}

	// P1: k1s0.outbox_message テーブルに INSERT する（Debezium CDC 経由で Kafka に転送される）
	// P2: この INSERT が失敗した場合は ErrOutboxInsertFailed を返し、呼び出し元が Rollback する
	const outboxSQL = `
		INSERT INTO k1s0.outbox_message
			(id, aggregate_id, tenant_id, event_kind, payload, created_at)
		VALUES
			($1, $2, current_setting('app.tenant_id')::uuid, 'OutboxRelay', $3::jsonb, $4)
	`
	// outbox INSERT を tx 内で実行する（P2 の rollback 要件を満たす）
	if _, err := tx.ExecContext(ctx, outboxSQL,
		// outbox エントリの ID をバインドする
		outboxID,
		// 変更対象の aggregate ID をバインドする
		change.AggregateID,
		// ペイロードを jsonb 文字列としてバインドする（PII は redact 済みのみ含む）
		change.Payload,
		// 書込完了日時をバインドする
		committedAt,
	); err != nil {
		// P2: outbox INSERT 失敗は ErrOutboxInsertFailed にマッピングして rollback を促す
		return nil, fmt.Errorf("%w: %v", ErrOutboxInsertFailed, err)
	}

	// P1+P4: k1s0.audit_event テーブルに INSERT する（全操作を監査記録する）
	// P4: pii_segregated は pgaudit も併用するが、アプリ層からも必ず audit_event を書く
	const auditEventSQL = `
		INSERT INTO k1s0.audit_event
			(id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)
		VALUES
			($1, $2, current_setting('app.tenant_id')::uuid,
			 current_setting('app.actor_id'),
			 current_setting('app.purpose'),
			 $3, $4::jsonb, $5)
	`
	// audit_event INSERT を tx 内で実行する（P4 の audit 必須要件を満たす）
	if _, err := tx.ExecContext(ctx, auditEventSQL,
		// audit_event の ID をバインドする（domain_event と同じ ID で結びつける）
		auditEventID,
		// 変更対象の aggregate ID をバインドする
		change.AggregateID,
		// テーブルクラスを文字列としてバインドする
		change.TableClass.String(),
		// ペイロードを jsonb 文字列としてバインドする
		change.Payload,
		// 書込完了日時をバインドする
		committedAt,
	); err != nil {
		// P4: audit_event INSERT 失敗は ErrPiiAuditFailed にマッピングして rollback を促す
		return nil, fmt.Errorf("%w: %v", ErrPiiAuditFailed, err)
	}

	// 三表書込の結果を返す（呼び出し元が tx.Commit() を呼ぶことで確定する）
	return &TripleWriteResult{
		// 書込んだ aggregate ID を返す
		AggregateID: change.AggregateID,
		// 書込んだ outbox エントリの ID を返す
		OutboxID: outboxID,
		// 書込んだ audit_event の ID を返す
		AuditEventID: auditEventID,
		// 書込完了日時を返す
		CommittedAt: committedAt,
	}, nil
}
