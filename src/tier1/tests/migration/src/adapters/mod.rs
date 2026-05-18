// adapters/mod.rs — MigrationAdapter trait + 4 pair 具象 adapter の共通定義と export
// spec 02 §migration_pair: CNPG↔StackGres / Strimzi↔RedPanda / Temporal↔internal / zen_rule↔internal
// 各 adapter は MigrationPhase と MigrationAdapter trait を本モジュールから import する

// anyhow: エラーハンドリング用クレートをインポートする
use anyhow::Result;

// ============================================================
// 共通 Phase 定義
// ============================================================

// 5 migration phase を表す列挙型（docs 02_移行Pair適合仕様 §phase 名に準拠）
#[derive(Debug, Clone, PartialEq)]
pub enum MigrationPhase {
    // フェーズ 1: スキーマ差分を計算して移行可能性を検証する
    SchemaDiff,
    // フェーズ 2: 移行元のデータを移行先にレプリケートする（ダウンタイムなし）
    StateReplicate,
    // フェーズ 3: 移行元・移行先への書込割合を段階的に増やす
    DualWriteRamp,
    // フェーズ 4: トラフィックを移行先に切り替える
    Cutover,
    // フェーズ 5: 問題が発生した場合に移行元に戻す
    Rollback,
}

// ============================================================
// 共通結果型
// ============================================================

// PhaseResult は各 phase の実行結果を表す構造体
#[derive(Debug)]
pub struct PhaseResult {
    // 実行が成功したかを示すフラグ
    pub passed: bool,
    // 詳細メッセージ
    pub message: String,
}

// ============================================================
// MigrationAdapter trait
// ============================================================

// MigrationAdapter は migration pair の phase 実行インターフェースを定義するトレイト
// 各 adapter 実装はこのトレイトを実装して phase 固有の処理を提供する
pub trait MigrationAdapter {
    // phase を受け取り実行結果を返すメソッド（同期 I/O でモックするため同期シグネチャを採用する）
    fn run_phase(&self, phase: MigrationPhase) -> Result<PhaseResult>;
}

// ============================================================
// 4 pair 具象 adapter モジュールの export
// ============================================================

// cnpg_stackgres: CNPG → StackGres PostgreSQL migration adapter
pub mod cnpg_stackgres;
// strimzi_redpanda: Strimzi Kafka → RedPanda migration adapter
pub mod strimzi_redpanda;
// temporal_internal: Temporal → internal workflow engine migration adapter
pub mod temporal_internal;
// zen_rule_internal: zen_rule → internal rule engine migration adapter
pub mod zen_rule_internal;

// ============================================================
// adapter モジュールから再エクスポートする
// ============================================================

// CNPG → StackGres adapter を公開する
pub use cnpg_stackgres::CnpgStackgresAdapter;
// Strimzi → RedPanda adapter を公開する
pub use strimzi_redpanda::StrimziRedpandaAdapter;
// Temporal → internal workflow adapter を公開する
pub use temporal_internal::TemporalInternalAdapter;
// zen_rule → internal rule adapter を公開する
pub use zen_rule_internal::ZenRuleInternalAdapter;
