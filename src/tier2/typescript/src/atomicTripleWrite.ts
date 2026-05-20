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
// pg PoolClient を import する（実 transaction を実行する接続クライアント）
import type { PoolClient } from "pg";
// HlcClock / HlcTimestamp を import する（wall-clock TTL 禁止規律: new Date() の代替）
import { HlcClock, HlcTimestamp } from "@k1s0/hlc-lib";

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
  // 書込完了 HLC タイムスタンプ（wall-clock TTL 禁止規律に従い HlcTimestamp を使用する）
  readonly committedAt: HlcTimestamp;
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
 * pg.PoolClient を受け取る execute メソッドを持つ
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
   * P1-P4: atomic 三表書込を実行する非同期メソッド（pg.PoolClient を使用する）
   * client: 呼び出し元が BEGIN した pg.PoolClient を受け取る
   * 呼び出し元は Ok 返却後に client.query('COMMIT') を呼ぶ。例外時は ROLLBACK を呼ぶ。
   * P1: state_change / outbox / audit_event の 3 INSERT を同一 txn で実行する
   * P2: outbox INSERT が失敗した場合は OutboxInsertFailedError をスローし、呼び出し元が ROLLBACK する
   * P3: tenant_id が GUC と一致しない場合は即座に TenantIdMismatchError をスローする
   * P4: pii_segregated テーブルへのアクセスは audit_event に記録してから txn を実行する
   */
  // execute メソッド（P1-P4 の実 transaction 実行）
  async execute(change: StateChange, client: PoolClient): Promise<TripleWriteResult> {
    // P3: tenant_id 一致を事前検証する（GUC と aggregate 行の tenant_id が一致しない場合は即座にエラー）
    this.verifyTenantId(change);
    // P4: pii_segregated の場合は audit_event への記録が必須であることを確認する
    void this.verifyPiiAuditRequired(change);

    // P3: SET LOCAL で 4 GUC を txn スコープに注入する（RLS FORCE が参照する）
    const setGucSql = this.#context.toSetLocalSql();
    // SET LOCAL GUC を client で実行する（transaction スコープのみ有効）
    await client.query(setGucSql);

    // outbox エントリの ID を生成する（P1 の atomic 三表書込で使用する）
    const outboxId = crypto.randomUUID();
    // audit_event の ID を生成する（domain_event と audit_event で共有する）
    const auditEventId = crypto.randomUUID();
    // HLC クロックを生成する（wall-clock TTL 禁止規律: new Date() の代替）
    const hlcClock = HlcClock.fromEnv();
    // HLC タイムスタンプを取得する（3 INSERT で統一した論理時刻を使用する）
    const committedAt: HlcTimestamp = hlcClock.now();
    // DB への bind 用に HLC の wall_ms から ISO 8601 文字列を生成する（DB 列型は TIMESTAMPTZ）
    const committedAtDb = new Date(Number(committedAt.wall_ms)).toISOString();

    // P1: k1s0.domain_event テーブルに INSERT する（aggregate 状態変更の永続化）
    // current_setting('app.tenant_id')::uuid を使って RLS FORCE の tenant_id を注入する
    await client.query(
      // parameterized query でバインドする（SQL injection を物理的に防ぐ）
      `INSERT INTO k1s0.domain_event
         (id, aggregate_id, tenant_id, event_kind, payload, version, created_at)
       VALUES
         ($1, $2, current_setting('app.tenant_id')::uuid, 'StateChange', $3::jsonb, $4, $5)`,
      [
        // audit_event_id を domain_event の主キーとして使用する
        auditEventId,
        // 変更対象の aggregate ID をバインドする
        change.aggregateId,
        // ペイロードを jsonb 文字列としてバインドする
        change.payload,
        // aggregate バージョンをバインドする（楽観的ロックに使用する）
        change.version,
        // HLC wall_ms から変換した ISO 8601 文字列をバインドする
        committedAtDb,
      ],
    );

    // P1: k1s0.outbox_message テーブルに INSERT する（Debezium CDC 経由で Kafka に転送される）
    // P2: この INSERT が失敗した場合は OutboxInsertFailedError をスローし、呼び出し元が ROLLBACK する
    // migration SoT: 0001_initial_schema.sql が CREATE TABLE k1s0.outbox_message を発行している
    try {
      // parameterized query で outbox_message INSERT を実行する
      await client.query(
        `INSERT INTO k1s0.outbox_message
           (id, aggregate_id, tenant_id, event_kind, payload, created_at)
         VALUES
           ($1, $2, current_setting('app.tenant_id')::uuid, 'OutboxRelay', $3::jsonb, $4)`,
        [
          // outbox エントリの ID をバインドする
          outboxId,
          // 変更対象の aggregate ID をバインドする
          change.aggregateId,
          // ペイロードを jsonb 文字列としてバインドする（PII は redact 済みのみ含む）
          change.payload,
          // HLC wall_ms から変換した ISO 8601 文字列をバインドする
          committedAtDb,
        ],
      );
    } catch (err) {
      // P2: outbox INSERT 失敗は OutboxInsertFailedError にマッピングして rollback を促す
      throw new OutboxInsertFailedError(
        err instanceof Error ? err.message : String(err),
      );
    }

    // P1+P4: k1s0.audit_event テーブルに INSERT する（全操作を監査記録する）
    // P4: pii_segregated は pgaudit も併用するが、アプリ層からも必ず audit_event を書く
    await client.query(
      // parameterized query で audit_event INSERT を実行する
      `INSERT INTO k1s0.audit_event
         (id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)
       VALUES
         ($1, $2, current_setting('app.tenant_id')::uuid,
          current_setting('app.actor_id'),
          current_setting('app.purpose'),
          $3, $4::jsonb, $5)`,
      [
        // audit_event の ID をバインドする（domain_event と同じ ID で結びつける）
        auditEventId,
        // 変更対象の aggregate ID をバインドする
        change.aggregateId,
        // テーブルクラスを文字列としてバインドする
        change.tableClass,
        // ペイロードを jsonb 文字列としてバインドする
        change.payload,
        // HLC wall_ms から変換した ISO 8601 文字列をバインドする
        committedAtDb,
      ],
    );

    // 三表書込の結果を返す（呼び出し元が COMMIT を呼ぶことで確定する）
    return {
      // 書込んだ aggregate ID を返す
      aggregateId: change.aggregateId,
      // 書込んだ outbox エントリの ID を返す
      outboxId,
      // 書込んだ audit_event の ID を返す
      auditEventId,
      // 書込完了 HLC タイムスタンプを返す
      committedAt,
    };
  }
}
