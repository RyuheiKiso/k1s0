// 本ファイルは k1s0-tier1-proto-gen crate のルート。
// buf generate（buf.gen.internal.yaml）が ./src/ 配下に出力する平坦な .rs を、
// Rust の module 階層 k1s0::internal::v1 に束ねる。
//
// docs 正典:
//   docs/05_実装/20_コード生成設計/10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md
// crate 全体で警告を error にしない（生成物の deprecated アトリビュート許容）
#![allow(clippy::all, rustdoc::all)]

// k1s0 名前空間（最上位）
pub mod k1s0 {
    // tier1 内部 gRPC の internal namespace（tier2/tier3 不可視）
    pub mod internal {
        // バージョン v1
        pub mod v1 {
            // prost 生成物の include。tonic 生成は本リリース時点では placeholder.proto に
            // service が無いため tonic 出力はゼロ件（include する .tonic.rs は存在しない）。
            include!("./k1s0.internal.v1.rs");
        }
    }
}
