// proto 生成コードのインクルード。

pub mod k1s0 {
    pub mod system {
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
