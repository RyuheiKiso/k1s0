pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
    }
    pub mod service {
        pub mod payment {
            pub mod v1 {
                include!("k1s0.service.payment.v1.rs");
            }
        }
    }
    // Kafka publish 用イベントメッセージの Protobuf 生成コード
    pub mod event {
        pub mod service {
            pub mod payment {
                pub mod v1 {
                    include!("k1s0.event.service.payment.v1.rs");
                }
            }
        }
    }
}
