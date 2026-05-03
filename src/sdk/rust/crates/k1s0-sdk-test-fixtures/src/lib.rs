//! k1s0 Rust SDK と同梱される e2e test 用 fixture ライブラリ。
//!
//! # 設計正典
//!
//! - ADR-TEST-010（test-fixtures 4 言語 SDK 同梱）
//! - `docs/05_実装/30_CI_CD設計/35_e2e_test_design/30_test_fixtures/01_4言語対称API.md`
//!
//! # 利用者の使い方
//!
//! ```ignore
//! use k1s0_sdk_test_fixtures::{setup, Options, Stack};
//!
//! #[tokio::test]
//! async fn my_test() {
//!     let fx = setup(Options { kind_nodes: 2, stack: Stack::Minimum, ..Default::default() })
//!         .await
//!         .unwrap();
//!     // 利用者の test code
//!     fx.teardown().await;
//! }
//! ```
//!
//! # バージョニング
//!
//! 本 crate は k1s0-sdk と同 workspace / 同 SemVer / 同 release tag で出る
//! （ADR-TEST-010 §2 versioning）。

// 各モジュールを公開
pub mod options;
pub mod fixture;
pub mod mock_builder;
pub mod wait_assert;

// よく使う型を re-export（利用者の use を短縮）
pub use options::{Options, Stack};
pub use fixture::{setup, Fixture};
pub use mock_builder::{MockBuilderRoot, StateMockBuilder, AuditMockBuilder, PubSubMockBuilder};

// crate 全体の Result 型 alias（thiserror で error 型を集約）
pub type Result<T> = std::result::Result<T, FixtureError>;

/// Fixture 操作で発生する error 型
#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    /// kind cluster 起動失敗
    #[error("kind cluster setup failed: {0}")]
    KindSetup(String),

    /// SDK client init 失敗
    #[error("SDK client init failed: {0}")]
    SdkClient(String),

    /// readiness wait timeout
    #[error("[k1s0-test-fixtures] WaitFor {resource:?} timeout after {seconds}s")]
    WaitTimeout {
        // 待機対象 resource 名（4 言語共通フォーマット）
        resource: String,
        // timeout 秒数
        seconds: u64,
    },

    /// 採用初期で実装する builder の panic
    #[error("ADR-TEST-010 PHASE: {service} mock builder は{phase}で実装、リリース時点未対応")]
    Unimplemented {
        // service 名
        service: String,
        // 実装予定 phase
        phase: String,
    },
}
