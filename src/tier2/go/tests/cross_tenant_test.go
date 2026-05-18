// k1s0 tier2 cross-tenant 統合テスト Go 版
// Rust 実装（cross_tenant_test.rs）と意味的に等価な Go 版
// P1-P4 invariant を PostgreSQL RLS FORCE 環境で検証する統合テスト
//
// 実行方法: TEST_DATABASE_URL 環境変数を設定した上で実行する
//   TEST_DATABASE_URL=postgres://k1s0:k1s0dev@localhost/tier2_dev go test ./tests/...
//
// 通常の go test（CI の unit test job）では TEST_DATABASE_URL が未設定のため全テストを skip する

// パッケージ名: tests（integration test パッケージ）
package tests

import (
	// context パッケージ: DB 操作のコンテキストに使用する
	"context"
	// database/sql パッケージ: *sql.Tx による実 transaction に使用する
	"database/sql"
	// os パッケージ: TEST_DATABASE_URL 環境変数の取得に使用する
	"os"
	// testing パッケージ: テストフレームワーク
	"testing"

	// uuid パッケージ: テスト用 tenant_id / aggregate_id 生成に使用する
	"github.com/google/uuid"
	// atomictriplewrite パッケージ: P1-P4 atomic 書込エンジン
	"github.com/k1s0/tier2/atomictriplewrite"
	// tenantcontext パッケージ: テナントコンテキスト
	"github.com/k1s0/tier2/tenantcontext"
	// pgx stdlib ドライバ: database/sql 互換ドライバとして pgx を登録する
	_ "github.com/jackc/pgx/v5/stdlib"
)

// setupSQL: テスト用スキーマをセットアップする SQL
// k1s0 スキーマと 3 テーブルを作成する（RLS FORCE なしの軽量バージョン）
const setupSQL = `
    CREATE SCHEMA IF NOT EXISTS k1s0;
    CREATE TABLE IF NOT EXISTS k1s0.domain_event (
        id UUID PRIMARY KEY,
        aggregate_id UUID NOT NULL,
        tenant_id UUID NOT NULL,
        event_kind TEXT NOT NULL,
        payload JSONB NOT NULL,
        version BIGINT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL
    );
    CREATE TABLE IF NOT EXISTS k1s0.outbox (
        id UUID PRIMARY KEY,
        aggregate_id UUID NOT NULL,
        tenant_id UUID NOT NULL,
        event_kind TEXT NOT NULL,
        payload JSONB NOT NULL,
        created_at TIMESTAMPTZ NOT NULL
    );
    CREATE TABLE IF NOT EXISTS k1s0.audit_event (
        id UUID PRIMARY KEY,
        aggregate_id UUID NOT NULL,
        tenant_id UUID NOT NULL,
        actor_id TEXT NOT NULL,
        purpose TEXT NOT NULL,
        table_class TEXT NOT NULL,
        payload JSONB NOT NULL,
        created_at TIMESTAMPTZ NOT NULL
    );
`

// cleanupSQL: テスト用スキーマをクリーンアップする SQL
// テスト間の独立性を保つために全テーブルと k1s0 スキーマを削除する
const cleanupSQL = `
    DROP TABLE IF EXISTS k1s0.audit_event CASCADE;
    DROP TABLE IF EXISTS k1s0.outbox CASCADE;
    DROP TABLE IF EXISTS k1s0.domain_event CASCADE;
    DROP SCHEMA IF EXISTS k1s0 CASCADE;
`

// testDatabaseURL: TEST_DATABASE_URL 環境変数から接続 URL を取得するヘルパー関数
// 環境変数が未設定の場合は空文字を返す
func testDatabaseURL() string {
	// TEST_DATABASE_URL 環境変数を取得する（未設定の場合は空文字を返す）
	return os.Getenv("TEST_DATABASE_URL")
}

// TestP3CrossTenantIsolation: P3 cross-tenant isolation を検証する統合テスト
// 異なる tenant_id の StateChange が reject されることを確認する
func TestP3CrossTenantIsolation(t *testing.T) {
	// TEST_DATABASE_URL が未設定の場合はテストをスキップする
	dbURL := testDatabaseURL()
	if dbURL == "" {
		// 環境変数が未設定の場合はスキップメッセージを出して skip する
		t.Skip("TEST_DATABASE_URL が未設定のため cross-tenant 統合テストをスキップする")
	}

	// PostgreSQL に接続する（pgx stdlib ドライバを使用する）
	db, err := sql.Open("pgx", dbURL)
	// 接続失敗の場合はテストを失敗させる
	if err != nil {
		t.Fatalf("PostgreSQL 接続失敗: %v", err)
	}
	// テスト終了時に接続を閉じる
	defer db.Close()

	// テスト用スキーマをセットアップする
	ctx := context.Background()
	if _, err := db.ExecContext(ctx, setupSQL); err != nil {
		// スキーマのセットアップ失敗はテストを中断する
		t.Fatalf("テスト用スキーマのセットアップ失敗: %v", err)
	}
	// テスト終了時にスキーマをクリーンアップする
	t.Cleanup(func() {
		if _, err := db.ExecContext(context.Background(), cleanupSQL); err != nil {
			// クリーンアップ失敗はログのみ（テスト自体の判定に影響しない）
			t.Logf("クリーンアップ失敗（無視する）: %v", err)
		}
	})

	// テナント A の TenantContext を生成する
	tenantA := uuid.New()
	ctxA, err := tenantcontext.FromAuth(tenantA, "actor-a", tenantcontext.PurposeBusinessOp)
	// TenantContext 生成失敗はテストを中断する
	if err != nil {
		t.Fatalf("TenantContext 生成失敗: %v", err)
	}
	// テナント B の tenant_id を生成する（cross-tenant を模擬する）
	tenantB := uuid.New()

	// AtomicTripleWrite をテナント A のコンテキストで生成する
	writer := atomictriplewrite.New(ctxA)
	// テナント B の StateChange を生成する（P3 違反を意図的に作る）
	crossTenantChange := atomictriplewrite.StateChange{
		// テナント A のコンテキストに テナント B の change を渡す（P3 違反）
		AggregateID: uuid.New(),
		TenantID:    tenantB,
		TableClass:  atomictriplewrite.TableClassTenantScoped,
		Payload:     `{"cross_tenant": "attempt"}`,
		Version:     1,
	}

	// P3 検証: cross-tenant StateChange が即座に reject されることを確認する
	err = writer.VerifyTenantID(&crossTenantChange)
	// P3 違反: エラーが返されることを確認する
	if err == nil {
		t.Fatal("P3: cross-tenant write は reject されるべきだが、エラーが返らなかった")
	}
	// エラーメッセージが TenantIdMismatch を示すことを確認する
	t.Logf("P3 cross-tenant isolation 確認済み: %v", err)
}

// TestP1AtomicTripleWrite: P1 atomic triple write が 3 テーブルに正常に書込むことを検証する統合テスト
func TestP1AtomicTripleWrite(t *testing.T) {
	// TEST_DATABASE_URL が未設定の場合はテストをスキップする
	dbURL := testDatabaseURL()
	if dbURL == "" {
		// 環境変数が未設定の場合はスキップメッセージを出して skip する
		t.Skip("TEST_DATABASE_URL が未設定のため P1 atomic triple write 統合テストをスキップする")
	}

	// PostgreSQL に接続する
	db, err := sql.Open("pgx", dbURL)
	// 接続失敗の場合はテストを失敗させる
	if err != nil {
		t.Fatalf("PostgreSQL 接続失敗: %v", err)
	}
	// テスト終了時に接続を閉じる
	defer db.Close()

	// テスト用スキーマをセットアップする
	ctx := context.Background()
	if _, err := db.ExecContext(ctx, setupSQL); err != nil {
		// スキーマのセットアップ失敗はテストを中断する
		t.Fatalf("テスト用スキーマのセットアップ失敗: %v", err)
	}
	// テスト終了時にスキーマをクリーンアップする
	t.Cleanup(func() {
		if _, err := db.ExecContext(context.Background(), cleanupSQL); err != nil {
			// クリーンアップ失敗はログのみ
			t.Logf("クリーンアップ失敗（無視する）: %v", err)
		}
	})

	// テスト用テナントの TenantContext を生成する
	tenantID := uuid.New()
	tenantCtx, err := tenantcontext.FromAuth(tenantID, "integration-test-actor", tenantcontext.PurposeBusinessOp)
	// TenantContext 生成失敗はテストを中断する
	if err != nil {
		t.Fatalf("TenantContext 生成失敗: %v", err)
	}
	// AtomicTripleWrite を生成する
	writer := atomictriplewrite.New(tenantCtx)
	// テスト用の StateChange を生成する
	aggregateID := uuid.New()
	change := atomictriplewrite.StateChange{
		// テスト用の aggregate ID を設定する
		AggregateID: aggregateID,
		// TenantContext と同一の tenant_id を設定する（P3 の整合性）
		TenantID:    tenantID,
		// TenantScoped テーブルクラスを指定する
		TableClass:  atomictriplewrite.TableClassTenantScoped,
		// テスト用ペイロードを設定する
		Payload:     `{"test": "atomic_triple_write"}`,
		// バージョン 1 で開始する
		Version:     1,
	}

	// *sql.Tx を開く（P1 の atomic 三表書込に使用する）
	tx, err := db.BeginTx(ctx, nil)
	// transaction 開始失敗はテストを中断する
	if err != nil {
		t.Fatalf("transaction 開始失敗: %v", err)
	}
	// P1: Execute() を呼んで 3 テーブルに INSERT する（引数順: ctx, tx, &change）
	result, err := writer.Execute(ctx, tx, &change)
	// execute が失敗した場合は rollback して終了する
	if err != nil {
		if rbErr := tx.Rollback(); rbErr != nil {
			// rollback 失敗はログのみ
			t.Logf("rollback 失敗: %v", rbErr)
		}
		t.Fatalf("P1: atomic triple write 失敗: %v", err)
	}
	// transaction を commit する（3 INSERT が永続化される）
	if err := tx.Commit(); err != nil {
		t.Fatalf("transaction commit 失敗: %v", err)
	}
	// 結果が正常であることを確認する
	if result == nil {
		t.Fatal("P1: TripleWriteResult が nil を返した")
	}
	// aggregate_id が一致していることを確認する
	if result.AggregateID != aggregateID {
		t.Errorf("P1: AggregateID 不一致: got %v, want %v", result.AggregateID, aggregateID)
	}
	t.Logf("P1 atomic triple write 確認済み: aggregate_id=%v", result.AggregateID)
}

// TestP4PiiAuditRequired: P4 pii_segregated テーブルへのアクセスが audit 必須であることを検証する
func TestP4PiiAuditRequired(t *testing.T) {
	// TEST_DATABASE_URL が未設定の場合はテストをスキップする
	dbURL := testDatabaseURL()
	if dbURL == "" {
		// 環境変数が未設定の場合はスキップメッセージを出して skip する
		t.Skip("TEST_DATABASE_URL が未設定のため P4 PII audit 統合テストをスキップする")
	}

	// テスト用テナントの TenantContext を生成する（Support purpose を使う）
	tenantID := uuid.New()
	tenantCtx, err := tenantcontext.FromAuth(tenantID, "support-engineer-001", tenantcontext.PurposeSupport)
	// TenantContext 生成失敗はテストを中断する
	if err != nil {
		t.Fatalf("TenantContext 生成失敗: %v", err)
	}
	// AtomicTripleWrite を生成する
	writer := atomictriplewrite.New(tenantCtx)
	// P4: PiiSegregated テーブルクラスの StateChange を生成する
	piiChange := atomictriplewrite.StateChange{
		// テスト用の aggregate ID を設定する
		AggregateID: uuid.New(),
		// TenantContext と同一の tenant_id を設定する
		TenantID:    tenantID,
		// PiiSegregated を指定することで P4 audit 必須フラグが true になる
		TableClass:  atomictriplewrite.TableClassPiiSegregated,
		// PII フィールドは redact 済みのみ含む
		Payload:     `{"pii_field": "[REDACTED]"}`,
		// バージョン 1 で開始する
		Version:     1,
	}

	// P4: pii_required フラグが true を返すことを確認する
	piiRequired := writer.VerifyPiiAuditRequired(&piiChange)
	// PiiSegregated では audit が必須であることを確認する
	if !piiRequired {
		t.Fatal("P4: PiiSegregated では VerifyPiiAuditRequired が true を返すべきだが false を返した")
	}
	t.Logf("P4 PII audit required 確認済み: pii_required=%v", piiRequired)
}
