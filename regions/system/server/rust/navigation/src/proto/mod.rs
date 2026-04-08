pub mod k1s0 {
    pub mod system {
        pub mod navigation {
            // HIGH-001 監査対応: 生成済み proto コードの Clippy 警告を抑制する
            #[allow(clippy::default_trait_access, clippy::trivially_copy_pass_by_ref)]
            pub mod v1 {
                include!("k1s0.system.navigation.v1.rs");
            }
        }
    }
}
