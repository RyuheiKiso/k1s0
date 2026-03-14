// proto生成コードをインクルード。
// prost-build (tonic-build) によって生成されたファイルを使用。

pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                // 共通protoが生成された場合にインクルード
                // include!("k1s0.system.common.v1.rs");
            }
        }
        pub mod ai_gateway {
            pub mod v1 {
                // AI Gateway protoが生成された場合にインクルード
                // include!("k1s0.system.ai_gateway.v1.rs");
            }
        }
    }
}
