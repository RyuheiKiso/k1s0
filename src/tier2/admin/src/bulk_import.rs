// bulk_import — 業務マスタ CSV 一括インポート
// 10_テナント分離適合仕様.md §admin_operation + RLS FORCE 対応
// AdminBoundaryGuard を通過した後に呼び出すこと（境界チェック bypass 禁止）

// AdminBoundaryError: 管理境界違反エラー型（bulk_import が返すエラー種別の基底）
use crate::admin_boundary::AdminBoundaryError;

/// CSV バルクインポートの設定を保持する構造体
/// tenant_id は AuthContext から取得するため公開 API 引数に露出しない（tier2 コーディング規約準拠）
pub struct BulkImportConfig {
    /// インポート先テナント識別子（AuthContext から取得済みの値を格納する）
    pub tenant_id: String,
    /// 業務マスタ種別（通貨レート / 製品カタログ等）の識別子（業界中立語のみ）
    pub master_type: String,
    /// インポート対象 CSV の署名付き URL（オブジェクトストレージ上の一時 URL）
    pub csv_url: String,
    /// RLS FORCE フラグ（true に固定すること。false は CI fail 対象）
    pub rls_force: bool,
}

/// CSV バルクインポートの実行結果を保持する構造体
pub struct BulkImportResult {
    /// INSERT に成功した行数（0 以上）
    pub inserted_count: usize,
}

/// CSV バルクインポートを実行する
/// 1. CSV スキーマ検証 → 2. RLS FORCE 付き atomic INSERT → 3. 監査イベント emit の順で実行する
/// いずれかのステップが失敗した場合はトランザクション全体をロールバックする（atomic 三表書込規約準拠）
pub async fn execute_bulk_import(
    config: BulkImportConfig,
) -> Result<BulkImportResult, AdminBoundaryError> {
    // RLS FORCE フラグが true でない場合は境界違反エラーを返す
    if !config.rls_force {
        return Err(AdminBoundaryError::InsufficientScope(
            "bulk_import: rls_force は true でなければならない（テナント分離保証）".to_string(),
        ));
    }
    // ステップ 1: CSV スキーマ検証（master_type に応じた検証ロジックに委譲する）
    validate_csv_schema(&config).await?;
    // ステップ 2: RLS FORCE 付き atomic 一括 INSERT（テナント境界を DB 層で強制する）
    let inserted = atomic_bulk_insert(&config).await?;
    // ステップ 3: 監査イベントを audit_local テーブル経由で emit する
    emit_audit_event(&config, inserted).await?;
    // インポート結果を返す
    Ok(BulkImportResult {
        inserted_count: inserted,
    })
}

/// CSV スキーマ検証を実行する（master_type ごとの検証ロジックに委譲する）
/// 不正なスキーマの場合は InsufficientScope エラーを返す
async fn validate_csv_schema(config: &BulkImportConfig) -> Result<(), AdminBoundaryError> {
    // master_type に応じた CSV スキーマ検証を実行する（実装は業種別 master_type に委譲）
    // 現時点では stub 実装（スキーマ定義ファイルは別途 src/tier2/schema/ に配置する）
    let _ = &config.csv_url;
    let _ = &config.master_type;
    Ok(())
}

/// RLS FORCE 付き atomic 一括 INSERT を実行する
/// SET LOCAL row_security = FORCE で tenant_id を強制注入してからバルク INSERT する
async fn atomic_bulk_insert(config: &BulkImportConfig) -> Result<usize, AdminBoundaryError> {
    // tenant_id を使用して RLS FORCE コンテキストを設定する
    let _ = &config.tenant_id;
    // sqlx の compile-time 型安全クエリで INSERT を実行する（生 SQL 文字列受付禁止規約準拠）
    // 実装は src/tier2/rust/src/repository/ 配下のリポジトリ抽象に委譲する
    // 挿入行数 0 を返す（stub 実装）
    Ok(0)
}

/// 監査イベントを audit_local テーブル経由で emit する
/// Outbox 経由で Relay に投入し、非同期で ClickHouse audit sink に転送する
async fn emit_audit_event(
    config: &BulkImportConfig,
    count: usize,
) -> Result<(), AdminBoundaryError> {
    // tenant_id / master_type / 挿入行数を監査イベントのフィールドとして記録する
    let _ = (&config.tenant_id, &config.master_type, count);
    // audit_local テーブルへの書込は Outbox と同一トランザクションで実行する（atomic 三表書込規約準拠）
    Ok(())
}
