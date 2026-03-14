//! ビルディングブロック統合クレート
//!
//! 各サブクレート（bb-core, bb-binding, bb-pubsub, bb-secretstore, bb-statestore）の
//! 型を一か所から参照できるようにまとめたファサードクレート。
//! 既存インポートとの後方互換性を維持する。

// コア型の再エクスポート（既存インポートとの後方互換）
pub mod component {
    pub use k1s0_bb_core::component::*;
}

pub mod config {
    pub use k1s0_bb_core::config::*;
}

pub mod error {
    pub use k1s0_bb_core::error::*;
}

pub mod registry {
    pub use k1s0_bb_core::registry::*;
}

// サブクレート型の統合アクセス用再エクスポート
pub mod binding {
    pub use k1s0_bb_binding::*;
}

pub mod pubsub {
    pub use k1s0_bb_pubsub::*;
}

pub mod secretstore {
    pub use k1s0_bb_secretstore::*;
}

pub mod statestore {
    pub use k1s0_bb_statestore::*;
}

// トップレベル再エクスポート
pub use component::{Component, ComponentStatus};
pub use config::{ComponentConfig, ComponentsConfig};
pub use error::ComponentError;
pub use registry::ComponentRegistry;
