pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
    }
    pub mod service {
        pub mod order {
            pub mod v1 {
                include!("k1s0.service.order.v1.rs");
            }
        }
    }
    // Kafka publish 用イベント Protobuf 定義
    pub mod event {
        pub mod service {
            pub mod order {
                pub mod v1 {
                    include!("k1s0.event.service.order.v1.rs");
                }
            }
            // Saga 補償: Order Consumer が購読する payment イベント型（C-001）
            pub mod payment {
                pub mod v1 {
                    include!("k1s0.event.service.payment.v1.rs");
                }
            }
        }
    }
}
