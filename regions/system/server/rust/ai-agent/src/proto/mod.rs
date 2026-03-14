// proto 生成コードをインクルード。
// prost-build (tonic-build) によって生成されたファイルを使用。

pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
        pub mod ai_agent {
            pub mod v1 {
                include!("k1s0.system.ai_agent.v1.rs");
            }
        }
    }
}
