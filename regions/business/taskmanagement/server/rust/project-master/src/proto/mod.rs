// gRPC 生成コードのモジュール構造。
// build.rs の tonic-build により src/proto/*.rs が生成される。
pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
    }
    pub mod business {
        pub mod taskmanagement {
            pub mod projectmaster {
                pub mod v1 {
                    include!("k1s0.business.taskmanagement.projectmaster.v1.rs");
                }
            }
        }
    }
}
