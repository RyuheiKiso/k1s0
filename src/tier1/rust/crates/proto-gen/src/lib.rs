// 本ファイルは k1s0-tier1-proto-gen crate のルート。
// buf generate（buf.gen.internal.yaml）が ./src/ 配下に出力する平坦な .rs を、
// Rust の module 階層 k1s0::internal::<comp>::v1 に束ねる。
//
// docs 正典:
//   docs/05_実装/20_コード生成設計/10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md
//   docs/04_概要設計/20_ソフトウェア方式設計/03_内部インタフェース方式設計/01_内部gRPC契約方式.md
//     - DS-SW-IIF-004（命名 k1s0.internal.<comp>.v<version>.<ServiceName>）
//
// crate 全体で警告を error にしない（生成物の deprecated アトリビュート許容）
#![allow(clippy::all, rustdoc::all)]

// k1s0 名前空間（最上位）
pub mod k1s0 {
    // tier1 内部 gRPC の internal namespace（tier2/tier3 不可視）
    pub mod internal {
        // 構造化エラー型 v1（DS-SW-IIF-012、ErrorDetail / ErrorCategory）
        pub mod errors {
            pub mod v1 {
                include!("./k1s0.internal.errors.v1.rs");
            }
        }
        // Audit core service v1（COMP-T1-AUDIT、AppendHash / VerifyChain）
        pub mod audit {
            pub mod v1 {
                include!("./k1s0.internal.audit.v1.rs");
                include!("./k1s0.internal.audit.v1.tonic.rs");
            }
        }
        // Decision core service v1（COMP-T1-DECISION、EvaluateDecision）
        pub mod decision {
            pub mod v1 {
                include!("./k1s0.internal.decision.v1.rs");
                include!("./k1s0.internal.decision.v1.tonic.rs");
            }
        }
        // PII Mask core service v1（COMP-T1-PII、MaskPii / MaskPiiBatch）
        pub mod pii {
            pub mod v1 {
                include!("./k1s0.internal.pii.v1.rs");
                include!("./k1s0.internal.pii.v1.tonic.rs");
            }
        }
    }
}
