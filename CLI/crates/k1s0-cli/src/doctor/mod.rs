//! k1s0 doctor - 環境診断モジュール
//!
//! 開発環境の健全性をチェックし、問題を診断する機能を提供する。

pub mod checker;
pub mod recommendation;
pub mod requirements;

pub use checker::{check_all_tools, check_tool, check_tools_by_category, CheckStatus, ToolCheck};
pub use recommendation::{
    generate_recommendations, generate_summary, has_optional_problems, has_required_problems,
    CheckSummary, RecommendAction, Recommendation,
};
pub use requirements::{ToolCategory, ToolRequirement};
