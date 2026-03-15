// proto 生成コードをインクルード。
// prost-build (tonic-build) によって生成されたファイルを使用。

pub mod k1s0 {
    pub mod system {
        #[allow(dead_code)]
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
        pub mod eventstore {
            pub mod v1 {
                include!("k1s0.system.eventstore.v1.rs");
            }
        }
    }
}
