// proto 生成コードのインクルード。

pub mod k1s0 {
    pub mod system {
        // HIGH-001 監査対応: tonic-build生成コードのClippy警告を抑制する
        #[allow(
            clippy::default_trait_access,
            clippy::trivially_copy_pass_by_ref,
            clippy::too_many_lines,
            clippy::doc_markdown
        )]
        pub mod config {
            pub mod v1 {
                include!("k1s0.system.config.v1.rs");
            }
        }
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
    }
}
