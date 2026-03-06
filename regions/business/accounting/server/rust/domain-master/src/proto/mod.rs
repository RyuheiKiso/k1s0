pub mod k1s0 {
    pub mod system {
        pub mod common {
            pub mod v1 {
                include!("k1s0.system.common.v1.rs");
            }
        }
    }
    pub mod business {
        pub mod accounting {
            pub mod domainmaster {
                pub mod v1 {
                    include!("k1s0.business.accounting.domainmaster.v1.rs");
                }
            }
        }
    }
}
