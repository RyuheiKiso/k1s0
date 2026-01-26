//! ドメイン層
//!
//! 設定管理のビジネスロジックを定義する。

pub mod entity;
pub mod repository;
pub mod error;

pub use entity::*;
pub use repository::*;
pub use error::*;
