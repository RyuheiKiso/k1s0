// proto 生成コードをインクルード。
// prost-build (tonic-build) によって生成されたファイルを使用。

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
        pub mod session {
            pub mod v1 {
                include!("k1s0.system.session.v1.rs");
            }
        }
    }
}
