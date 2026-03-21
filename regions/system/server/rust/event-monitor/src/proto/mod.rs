pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
        pub mod event_monitor {
            pub mod v1 {
                include!("k1s0.system.eventmonitor.v1.rs");
            }
        }
        // DLQ Manager サービスの gRPC クライアントコードを提供するモジュール。
        // build.rs で dlq.proto から生成される。
        pub mod dlq {
            pub mod v1 {
                include!("k1s0.system.dlq.v1.rs");
            }
        }
    }
}
