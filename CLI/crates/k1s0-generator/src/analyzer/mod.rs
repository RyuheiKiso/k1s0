//! プロジェクト分析モジュール
//!
//! 既存プロジェクトの構造、規約準拠、依存関係を分析し、
//! k1s0 への移行計画を生成する。

pub mod convention;
pub mod dependency;
pub mod detector;
pub mod env_convert;
pub mod plan;
pub mod scorer;
pub mod structure;
pub mod types;

pub use convention::scan_violations;
pub use dependency::analyze_dependencies;
pub use detector::detect_project_type;
pub use env_convert::{convert_env_to_config, parse_env_file};
pub use plan::generate_migration_plan;
pub use scorer::calculate_scores;
pub use structure::analyze_structure;
pub use types::*;
