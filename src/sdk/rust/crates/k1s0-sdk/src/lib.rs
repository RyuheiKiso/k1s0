// 本ファイルは k1s0-sdk crate のルート。tier1 公開 API への高水準ファサード。
//
// docs 正典:
//   docs/05_実装/10_ビルド設計/10_Rust_Cargo_workspace/01_Rust_Cargo_workspace.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/
//
// scope（リリース時点 最小、3 代表 service）:
//   - Client::state() → StateFacade（get / save / delete）
//   - Client::pubsub() → PubSubFacade（publish）
//   - Client::secrets() → SecretsFacade（get / rotate）
//   - Client::raw() → 12 service すべての生成 stub クライアントへの直接アクセス
// crate 全体で警告を error にしない（生成物の deprecated アトリビュート許容）
#![allow(clippy::all)]

// proto 生成物を `proto` 名で再 export する。
pub use k1s0_sdk_proto as proto;

// 公開 sub module（12 service すべて）。
pub mod audit;
pub mod binding;
pub mod client;
pub mod decision;
pub mod decision_admin;
pub mod feature;
pub mod feature_admin;
pub mod invoke;
pub mod log;
pub mod pii;
pub mod pubsub;
pub mod secrets;
pub mod state;
pub mod telemetry;
pub mod workflow;

// 主要型を crate ルートで再 export する（`use k1s0_sdk::Client;` で使えるように）。
pub use audit::AuditFacade;
pub use binding::BindingFacade;
pub use client::{Client, Config};
pub use decision::DecisionFacade;
pub use decision_admin::DecisionAdminFacade;
pub use feature::FeatureFacade;
pub use feature_admin::FeatureAdminFacade;
pub use invoke::InvokeFacade;
pub use log::LogFacade;
pub use pii::PiiFacade;
pub use pubsub::PubSubFacade;
pub use secrets::SecretsFacade;
pub use state::StateFacade;
pub use telemetry::TelemetryFacade;
pub use workflow::WorkflowFacade;
