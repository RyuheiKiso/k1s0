// 本ファイルは k1s0-sdk crate のルート。tier1 公開 API への利用者向けエントリ。
//
// docs 正典:
//   docs/05_実装/10_ビルド設計/10_Rust_Cargo_workspace/01_Rust_Cargo_workspace.md
//
// scope（リリース時点）:
//   k1s0-sdk-proto を `proto` 名で再 export し、`use k1s0_sdk::proto::tier1::state::v1`
//   のような短い import path を提供する薄いラッパとする。動詞統一の高水準 facade
//   （`k1s0_sdk::State::save(...)` 等）はロードマップ #8 で追加予定。
// crate 全体で警告を error にしない（生成物の deprecated アトリビュート許容）
#![allow(clippy::all)]

// proto 生成物を `proto` という名前で再 export する。
// 利用例: `use k1s0_sdk::proto::k1s0::tier1::state::v1::StateServiceClient;`
pub use k1s0_sdk_proto as proto;
