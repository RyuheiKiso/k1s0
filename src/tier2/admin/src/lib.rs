// k1s0-tier2-admin クレートのルートモジュール（設計方針 14 / 管理境界）
// AdminBoundaryGuard トレイトと AdminOperation 列挙型を公開 API として提供する

// 管理操作列挙型モジュール（AdminOperation の定義）
pub mod admin_operation;
// 管理境界ガードモジュール（AdminBoundaryGuard トレイトの定義）
pub mod admin_boundary;

// 公開型の再エクスポート（tier2-admin 公開 API 表面を最小化する）
pub use admin_operation::AdminOperation;
// AdminRequest / AdminResponse / AdminBoundaryGuard / AdminBoundaryError を公開する
pub use admin_boundary::{AdminBoundaryError, AdminBoundaryGuard, AdminRequest, AdminResponse};
