// bulk_import: 業務マスタ CSV 一括インポート（TypeScript 版）
// 10_テナント分離適合仕様.md §admin_operation + RLS FORCE 対応
// Rust 実装（admin/src/bulk_import.rs）と 4 言語等価強度を持つ TypeScript 版
// AdminBoundaryGuard を通過した後に呼び出すこと（境界チェック bypass 禁止）

/**
 * CSV バルクインポートの設定インターフェース
 * tenantId は AuthContext から取得するため API 引数に露出しない（tier2 コーディング規約準拠）
 */
export interface BulkImportConfig {
  /** インポート先テナント識別子（AuthContext から取得済みの値を格納する） */
  tenantId: string;
  /** 業務マスタ種別（通貨レート / 製品カタログ等）の識別子（業界中立語のみ） */
  masterType: string;
  /** インポート対象 CSV の署名付き URL（オブジェクトストレージ上の一時 URL） */
  csvUrl: string;
  /** RLS FORCE フラグ（true に固定すること。false は CI fail 対象） */
  rlsForce: boolean;
}

/**
 * CSV バルクインポートの実行結果インターフェース
 */
export interface BulkImportResult {
  /** INSERT に成功した行数（0 以上） */
  insertedCount: number;
}

/**
 * CSV バルクインポートを実行する非同期関数
 * 1. CSV スキーマ検証 → 2. RLS FORCE 付き atomic INSERT → 3. 監査イベント emit の順で実行する
 * いずれかのステップが失敗した場合はトランザクション全体をロールバックする（atomic 三表書込規約準拠）
 */
export async function executeBulkImport(
  config: BulkImportConfig,
): Promise<BulkImportResult> {
  // RLS FORCE フラグが true でない場合は境界違反エラーをスローする
  if (!config.rlsForce) {
    throw new Error(
      "bulk_import: rlsForce は true でなければならない（テナント分離保証）",
    );
  }
  // ステップ 1: CSV スキーマ検証（masterType に応じた検証ロジックに委譲する）
  await validateCsvSchema(config);
  // ステップ 2: RLS FORCE 付き atomic 一括 INSERT（テナント境界を DB 層で強制する）
  const inserted = await atomicBulkInsert(config);
  // ステップ 3: 監査イベントを audit_local テーブル経由で emit する
  await emitAuditEvent(config, inserted);
  // インポート結果を返す
  return { insertedCount: inserted };
}

/**
 * CSV スキーマ検証を実行する（stub 実装）
 * master_type に応じた検証ロジックに委譲する
 */
async function validateCsvSchema(config: BulkImportConfig): Promise<void> {
  // master_type に応じた CSV スキーマ検証を実行する（実装は業種別 masterType に委譲）
  void config.csvUrl;
  void config.masterType;
}

/**
 * RLS FORCE 付き atomic 一括 INSERT を実行する（stub 実装）
 * SET LOCAL row_security = FORCE で tenantId を強制注入してからバルク INSERT する
 */
async function atomicBulkInsert(config: BulkImportConfig): Promise<number> {
  // pg ドライバの compile-time 型安全クエリを使用する（生 SQL 文字列受付禁止規約準拠）
  void config.tenantId;
  // 挿入行数 0 を返す（stub 実装）
  return 0;
}

/**
 * 監査イベントを audit_local テーブル経由で emit する（stub 実装）
 * Outbox 経由で Relay に投入し、非同期で ClickHouse audit sink に転送する
 */
async function emitAuditEvent(
  config: BulkImportConfig,
  count: number,
): Promise<void> {
  // tenantId / masterType / 挿入行数を監査イベントのフィールドとして記録する
  void config.tenantId;
  void config.masterType;
  void count;
}
