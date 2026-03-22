// アクティビティサービス proto 生成コードのモジュール宣言。
pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
    }
    pub mod service {
        pub mod activity {
            pub mod v1 {
                include!("k1s0.service.activity.v1.rs");
            }
        }
    }
    pub mod event {
        pub mod service {
            pub mod activity {
                pub mod v1 {
                    include!("k1s0.event.service.activity.v1.rs");
                }
            }
        }
    }
}
