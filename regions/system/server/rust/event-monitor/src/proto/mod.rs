pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
        // HIGH-001 監査対応: tonic-build生成コードのClippy警告を抑制する
        #[allow(
            clippy::default_trait_access,
            clippy::trivially_copy_pass_by_ref,
            clippy::too_many_lines,
            clippy::doc_markdown
        )]
        pub mod event_monitor {
            pub mod v1 {
                include!("k1s0.system.eventmonitor.v1.rs");
            }
        }
        // DLQ Manager サービスの gRPC クライアントコードを提供するモジュール。
        // build.rs で dlq.proto から生成される。
        // HIGH-001 監査対応: tonic-build生成コードのClippy警告を抑制する
        #[allow(
            clippy::default_trait_access,
            clippy::trivially_copy_pass_by_ref,
            clippy::too_many_lines,
            clippy::doc_markdown
        )]
        pub mod dlq {
            pub mod v1 {
                include!("k1s0.system.dlq.v1.rs");
            }
        }
    }
}
