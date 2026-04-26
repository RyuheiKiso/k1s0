// 本ファイルは k1s0-sdk-proto crate のルート。buf generate（neoeinstein-prost /
// neoeinstein-tonic）が ./gen/v1/ 配下に出力する平坦な .rs ファイル群を、
// Rust の module 階層 k1s0::tier1::<api>::v1 に束ねる。
//
// docs 正典:
//   docs/05_実装/20_コード生成設計/10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md
//     - 生成先: src/sdk/rust/crates/k1s0-sdk-proto/src/gen/v1/
//
// 構成:
//   - prost の出力: <package>.rs（例: k1s0.tier1.state.v1.rs）→ message / enum 型を含む
//   - tonic の出力: <package>.v1.tonic.rs → gRPC client / server 型を含む
//   両者を同 module に include することで `state_v1::StateServiceClient` 等の
//   フラットな利用感を提供する。
// crate 全体で警告を error にしない（生成物の deprecated アトリビュート許容）
#![allow(clippy::all, rustdoc::all)]

// k1s0 名前空間（最上位）
pub mod k1s0 {
    // tier1 名前空間（公開 12 API + 共通型 + health）
    pub mod tier1 {
        // 共通型（TenantContext / ErrorDetail / K1s0ErrorCategory）
        pub mod common {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.common.v1.rs");
            }
        }

        // ServiceInvokeService（Invoke / InvokeStream）
        pub mod serviceinvoke {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.serviceinvoke.v1.rs");
                // tonic 生成物の include（client / server 型）
                include!("../gen/v1/k1s0.tier1.serviceinvoke.v1.tonic.rs");
            }
        }

        // StateService（Get / Set / Delete / BulkGet / Transact）
        pub mod state {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.state.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.state.v1.tonic.rs");
            }
        }

        // PubSubService（Publish / BulkPublish / Subscribe）
        pub mod pubsub {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.pubsub.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.pubsub.v1.tonic.rs");
            }
        }

        // SecretsService（Get / BulkGet / Rotate）
        pub mod secrets {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.secrets.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.secrets.v1.tonic.rs");
            }
        }

        // BindingService（Invoke）
        pub mod binding {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.binding.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.binding.v1.tonic.rs");
            }
        }

        // WorkflowService（Start / Signal / Query / Cancel / Terminate / GetStatus）
        pub mod workflow {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.workflow.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.workflow.v1.tonic.rs");
            }
        }

        // LogService（Send / BulkSend）
        pub mod log {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.log.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.log.v1.tonic.rs");
            }
        }

        // TelemetryService（EmitMetric / EmitSpan）
        pub mod telemetry {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.telemetry.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.telemetry.v1.tonic.rs");
            }
        }

        // DecisionService + DecisionAdminService
        pub mod decision {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.decision.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.decision.v1.tonic.rs");
            }
        }

        // AuditService
        pub mod audit {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.audit.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.audit.v1.tonic.rs");
            }
        }

        // PiiService
        pub mod pii {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.pii.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.pii.v1.tonic.rs");
            }
        }

        // FeatureService + FeatureAdminService
        pub mod feature {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.feature.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.feature.v1.tonic.rs");
            }
        }

        // HealthService（Liveness / Readiness）
        pub mod health {
            // バージョン v1
            pub mod v1 {
                // prost 生成物の include
                include!("../gen/v1/k1s0.tier1.health.v1.rs");
                // tonic 生成物の include
                include!("../gen/v1/k1s0.tier1.health.v1.tonic.rs");
            }
        }
    }
}
