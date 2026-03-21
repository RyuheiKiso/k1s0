pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
        pub mod rule_engine {
            pub mod v1 {
                // tonic-build は proto package 名 `k1s0.system.ruleengine.v1` から
                // `k1s0.system.ruleengine.v1.rs` を生成する（ruleengine はスネークケースなし）
                include!("k1s0.system.ruleengine.v1.rs");
            }
        }
    }
}
