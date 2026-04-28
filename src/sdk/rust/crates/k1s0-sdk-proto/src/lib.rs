// 本ファイルは k1s0-sdk-proto crate のルート。buf generate（neoeinstein-prost /
// neoeinstein-tonic）が src/gen/v1/ 配下に出力する平坦な .rs ファイル群を、
// Rust の module 階層 k1s0::tier1::<api>::v1 に束ねる。
//
// docs 正典:
//   docs/05_実装/20_コード生成設計/10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md
//     - 生成先: src/sdk/rust/crates/k1s0-sdk-proto/src/gen/v1/
//
// 構成:
//   - prost の出力: <package>.rs（例: k1s0.tier1.state.v1.rs）→ message / enum 型を含む
//   - tonic の出力: <package>.v1.tonic.rs → gRPC client / server 型を含む
//   neoeinstein-prost は生成 .rs の末尾で対応する .tonic.rs を `include!` で
//   自動取り込みするため、本 lib.rs では .rs だけを include すれば tonic 生成も
//   一緒に取り込まれる。.tonic.rs を別途 include すると `pub mod *_service_client`
//   等が二重定義され E0428 で fail するため、こちらでは include しない。
//
// crate 全体で警告を error にしない（生成物の deprecated アトリビュート許容）
#![allow(clippy::all, rustdoc::all)]

/// `buf build src/contracts/tier1 -o ...` で生成した tier1 の FileDescriptorSet（バイナリ proto）。
/// tonic-reflection の `Builder::register_encoded_file_descriptor_set` に渡すと、
/// gRPC reflection 経由で grpcurl 等のクライアントから service / method / message
/// メタを参照可能になる。Rust Pod（t1-decision / audit / pii）の main.rs で利用する。
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("./gen/tier1.descriptor.binpb");

// k1s0 名前空間（最上位）
pub mod k1s0 {
    // tier1 名前空間（公開 12 API + 共通型 + health）
    pub mod tier1 {
        // 共通型（TenantContext / ErrorDetail / K1s0ErrorCategory）
        pub mod common {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.common.v1.rs");
            }
        }

        // ServiceInvokeService（Invoke / InvokeStream）
        pub mod serviceinvoke {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.serviceinvoke.v1.rs");
            }
        }

        // StateService（Get / Set / Delete / BulkGet / Transact）
        pub mod state {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.state.v1.rs");
            }
        }

        // PubSubService（Publish / BulkPublish / Subscribe）
        pub mod pubsub {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.pubsub.v1.rs");
            }
        }

        // SecretsService（Get / BulkGet / Rotate）
        pub mod secrets {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.secrets.v1.rs");
            }
        }

        // BindingService（Invoke）
        pub mod binding {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.binding.v1.rs");
            }
        }

        // WorkflowService（Start / Signal / Query / Cancel / Terminate / GetStatus）
        pub mod workflow {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.workflow.v1.rs");
            }
        }

        // LogService（Send / BulkSend）
        pub mod log {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.log.v1.rs");
            }
        }

        // TelemetryService（EmitMetric / EmitSpan）
        pub mod telemetry {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.telemetry.v1.rs");
            }
        }

        // DecisionService + DecisionAdminService
        pub mod decision {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.decision.v1.rs");
            }
        }

        // AuditService
        pub mod audit {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.audit.v1.rs");
            }
        }

        // PiiService
        pub mod pii {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.pii.v1.rs");
            }
        }

        // FeatureService + FeatureAdminService
        pub mod feature {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.feature.v1.rs");
            }
        }

        // HealthService（Liveness / Readiness）
        pub mod health {
            pub mod v1 {
                include!("./gen/v1/k1s0.tier1.health.v1.rs");
            }
        }
    }
}
