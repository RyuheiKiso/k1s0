// HIGH-001 監査対応: proto生成コードをインクルード。
// prost-build (tonic-build) によって生成されたファイルを使用。

pub mod k1s0 {
    pub mod system {
        pub mod ai_agent {
            // HIGH-001 監査対応: tonic-build生成コードのClippy警告を抑制する
            #[allow(
                clippy::default_trait_access,
                clippy::trivially_copy_pass_by_ref,
                clippy::too_many_lines,
                clippy::doc_markdown,
                clippy::missing_docs_in_private_items
            )]
            pub mod v1 {
                // tonic-buildで生成されたai_agentサービスのコードをインクルード
                // protoのpackage名はk1s0.system.aiagent.v1なのでファイル名もそれに合わせる
                include!("k1s0.system.aiagent.v1.rs");
            }
        }
    }
}
