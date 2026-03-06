pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
        pub mod rule_engine {
            pub mod v1 {
                include!("k1s0.system.rule_engine.v1.rs");
            }
        }
    }
}
