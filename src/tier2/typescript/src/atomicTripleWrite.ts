/**
 * k1s0 tier2 atomic 三表書込 TypeScript 実装
 * Rust 実装（atomic_triple_write.rs）と semantic 等価な TypeScript 版
 * State change / Outbox / Audit event を同一 DB トランザクションで書く
 * TLA+ の P1-P4 invariant と double-bind する（src/formal/dafny/AtomicThreeTableWrite.dfy）
 *
 * P1: aggregate 状態変更時、必ず Outbox + Audit が同一 txn に書込
 * P2: Outbox 投入失敗時、aggregate 状態変更も rollback
 * P3: tenant_id が GUC と aggregate 行で一致しない場合、操作を reject
 * P4: pii_segregated の全アクセスを audit_event に記録
 */

// TenantContext を import する（4 言語等価強度の型を共有する）
import { TenantContext } from "./tenantContext.js";

/**
 * 書込対象テーブルクラス（10_テナント分離適合仕様.md の 4 class と一致する）
 */
// TableClass 型定義（文字列リテラル型で許容値を制限する）
export type TableClass =
  // tenant_scoped: tenant_id 必須、RLS FORCE
  | "TenantScoped"
  // tenant_master: tenant_id 必須、role 制限付き RLS FORCE
  | "TenantMaster"
  // platform_global: tenant_id 無し、RLS 無効
  | "PlatformGlobal"
  // pii_segregated: tenant_id 必須 + purpose check + pgaudit 全アクセス
  | "PiiSegregated";

/**
 * aggregate の状態変更を表す型（P1 の state_change に対応する）
 */
// StateChange インターフェース定義
export interface StateChange {
  // 変更対象の aggregate ID（UUID 文字列）
  readonly aggregateId: string;
  // 変更対象の tenant_id（TenantContext.tenantId と一致している必要がある）
  readonly tenantId: string;
  // テーブルクラス（どの class の table を書込むかを示す）
  readonly tableClass: TableClass;
  // 変更内容のシリアライズ済みペイロード（JSON 文字列）
  readonly payload: string;
  // aggregate バージョン（楽観的ロックに使用する）
  readonly version: number;
}

/**
 * atomic 三表書込の結果
 */
// TripleWriteResult インターフェース定義
export interface TripleWriteResult {
  // 書込んだ aggregate ID
  readonly aggregateId: string;
  // 書込んだ outbox エントリの ID
  readonly outboxId: string;
  // 書込んだ audit_event の ID
  readonly auditEventId: string;
  // 書込完了日時（ISO 8601 文字列）
  readonly committedAt: string;
}

/**
 * P3 違反: tenant_id 不一致エラー
 */
// TenantIdMismatchError クラス定義
export class TenantIdMismatchError extends Error {
  // GUC の tenant_id
  readonly gucTenantId: string;
  // aggregate 行の tenant_id
  readonly rowTenantId: string;

  // コンストラクタ（guc と row の tenant_id を受け取る）
  constructor(gucTenantId: string, rowTenantId: string) {
    // エラーメッセージを構築する
    super(`P3 TenantId mismatch: guc=${gucTenantId}, row=${rowTenantId}`);
    // エラー名を設定する
    this.name = "TenantIdMismatchError";
    // フィールドを初期化する
    this.gucTenantId = gucTenantId;
    this.rowTenantId = rowTenantId;
  }
}

/**
 * P2 違反: Outbox 投入失敗エラー（rollback が必要）
 */
// OutboxInsertFailedError クラス定義
export class OutboxInsertFailedError extends Error {
  // コンストラクタ（原因メッセージを受け取る）
  constructor(cause: string) {
    // エラーメッセージを構築する
    super(`Outbox insert failed, transaction rolled back: ${cause}`);
    // エラー名を設定する
    this.name = "OutboxInsertFailedError";
  }
}

/**
 * atomic 三表書込の実行エンジン
 * postgres.js の Transaction を受け取る execute メソッドを持つ
 * TenantContext を使って GUC 注入と tenant_id 検証を行う
 */
// AtomicTripleWrite クラス定義
export class AtomicTripleWrite {
  // テナントコンテキスト（GUC 注入・tenant_id 検証に使用する）
  readonly #context: TenantContext;

  // コンストラクタ（TenantContext を受け取る）
  constructor(context: TenantContext) {
    // TenantContext を格納する
    this.#context = context;
  }

  /**
   * P3: tenant_id 一致を検証する
   * StateChange の tenantId が TenantContext の tenantId と一致しない場合は例外をスローする
   */
  // verifyTenantId メソッド（P3 の DB 層対応）
  verifyTenantId(change: StateChange): void {
    // GUC の tenant_id と aggregate の tenant_id を比較する
    const gucTenantId = this.#context.tenantId;
    if (gucTenantId !== change.tenantId) {
      // P3 違反: tenant_id 不一致で例外をスローする
      throw new TenantIdMismatchError(gucTenantId, change.tenantId);
    }
  }

  /**
   * P4: pii_segregated アクセスが audit_event 必須かを返す
   */
  // verifyPiiAuditRequired メソッド（P4 の DB 層対応）
  verifyPiiAuditRequired(change: StateChange): boolean {
    // PiiSegregated の場合は必ず audit_event を記録する（true を返す）
    return change.tableClass === "PiiSegregated";
  }

  /**
   * P1: 三表書込に必要な SQL 文字列を生成する
   * BEGIN 〜 COMMIT の間に state_change / outbox / audit_event の 3 INSERT を含む
   */
  // buildTripleWriteSql メソッド（SQL 生成のみ、実際の DB 実行は execute() が担う）
  buildTripleWriteSql(change: StateChange): string {
    // P3: tenant_id 一致を事前検証する
    this.verifyTenantId(change);
    // outbox エントリの ID を生成する（crypto.randomUUID を使用する）
    const outboxId = crypto.randomUUID();
    // audit_event の ID を生成する
    const auditId = crypto.randomUUID();
    // 現在時刻を ISO 8601 形式で取得する
    const now = new Date().toISOString();
    // SET LOCAL GUC 注入 SQL を取得する（4 GUC 全て）
    const setGuc = this.#context.toSetLocalSql();
    // payload の single quote をエスケープする（SQL injection 対策）
    const escapedPayload = change.payload.replace(/'/g, "''");
    // P1: state_change + outbox + audit_event を BEGIN 〜 COMMIT の間に書く
    return [
      "BEGIN;",
      setGuc,
      "",
      "-- P1: state_change (aggregate テーブルへの書込)",
      `INSERT INTO k1s0.domain_event (id, aggregate_id, tenant_id, event_kind, payload, version, created_at)`,
      `VALUES ('${auditId}', '${change.aggregateId}', current_setting('app.tenant_id')::uuid, 'StateChange', '${escapedPayload}'::jsonb, ${change.version}, '${now}');`,
      "",
      "-- P1: outbox (Debezium CDC 経由で Kafka に転送される)",
      `INSERT INTO k1s0.outbox (id, aggregate_id, tenant_id, event_kind, payload, created_at)`,
      `VALUES ('${outboxId}', '${change.aggregateId}', current_setting('app.tenant_id')::uuid, 'OutboxRelay', '${escapedPayload}'::jsonb, '${now}');`,
      "",
      "-- P1 + P4: audit_event (全操作で記録、pii_segregated は pgaudit も併用)",
      `INSERT INTO k1s0.audit_event (id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)`,
      `VALUES ('${auditId}', '${change.aggregateId}', current_setting('app.tenant_id')::uuid, current_setting('app.actor_id'), current_setting('app.purpose'), '${change.tableClass}', '${escapedPayload}'::jsonb, '${now}');`,
      "",
      "COMMIT;",
    ].join("\n");
  }

  /**
   * P1-P4: atomic 三表書込を実行する非同期メソッド（型シグネチャのみ、実 txn は TODO）
   * TODO: postgres.js 統合時に sql Transaction を第 2 引数に追加する
   * P1: BEGIN 〜 COMMIT の中で state_change / outbox / audit_event の 3 INSERT を実行する
   * P2: outbox INSERT が失敗した場合は txn を rollback して OutboxInsertFailedError をスローする
   * P3: tenant_id が GUC と一致しない場合は即座に reject して TenantIdMismatchError をスローする
   * P4: pii_segregated テーブルへのアクセスは audit_event に記録してから txn を実行する
   */
  // execute メソッド（P1-P4 の型シグネチャ）
  async execute(change: StateChange): Promise<TripleWriteResult> {
    // P3: tenant_id 一致を事前検証する（GUC と aggregate 行の tenant_id が一致しない場合は即座にエラー）
    this.verifyTenantId(change);
    // P4: pii_segregated の場合は audit_event への記録が必須であることを確認する
    void this.verifyPiiAuditRequired(change);
    // P1: 三表書込 SQL を生成する（実際の txn 実行は TODO）
    // TODO: postgres.js の sql<...>`...` を使って 3 INSERT + SET LOCAL を同一 txn で実行する
    void this.buildTripleWriteSql(change);
    // P2: outbox INSERT が失敗した場合は rollback のためのエラーをスローする
    // TODO: postgres.js の txn.unsafe(sql) の失敗を OutboxInsertFailedError にマッピングする
    // 現時点では成功結果を構築して返す（実 DB 実行なし）
    const outboxId = crypto.randomUUID();
    // audit_event の ID を生成する
    const auditEventId = crypto.randomUUID();
    // 書込完了日時を ISO 8601 形式で記録する
    const committedAt = new Date().toISOString();
    // TripleWriteResult を返す（実 txn 統合前の型シグネチャ確認用）
    return {
      aggregateId: change.aggregateId,
      outboxId,
      auditEventId,
      committedAt,
    };
  }
}
