// 生成コマンドのモジュール構成。
// execute.rs を分割した各サブモジュールを統括する。

pub mod config_types;
pub mod conflict;
pub mod context;
pub mod execute;
pub mod infra_gen;
pub mod navigation;
pub mod paths;
pub mod post_process;
pub mod retry;
pub mod scaffold;
pub mod template;
pub mod types;

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;

// 公開 API の re-export
pub use conflict::{ensure_generate_targets_available, find_generate_conflicts_at};
pub use execute::{execute_generate, execute_generate_at, execute_generate_with_config};
pub use execute::{scan_placements, scan_placements_at};
pub use paths::build_output_path;
pub use types::*;
