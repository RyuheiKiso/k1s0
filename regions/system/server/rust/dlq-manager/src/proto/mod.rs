pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
        pub mod dlq {
            pub mod v1 {
                include!("k1s0.system.dlq.v1.rs");
            }
        }
    }
}
