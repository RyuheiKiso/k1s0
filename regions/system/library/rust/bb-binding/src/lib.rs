// bb-binding クレートのルートモジュール。
// バインディング機能（入力・出力）を提供するモジュール群を公開する。

// エラー型モジュール: バインディング操作で発生するエラーを定義する
pub mod error;
// HTTP バインディングモジュール: "http" フィーチャーが有効な場合のみコンパイルされる
#[cfg(feature = "http")]
pub mod http;
// インメモリバインディングモジュール: テスト・開発用のインメモリ実装を提供する
pub mod memory;
// トレイトモジュール: バインディングの抽象インターフェースを定義する
pub mod traits;

// BindingError をクレートのトップレベルから直接参照できるように再エクスポートする
pub use error::BindingError;
// HTTP バインディングは "http" フィーチャーが有効な場合のみ再エクスポートする
#[cfg(feature = "http")]
pub use http::HttpOutputBinding;
// インメモリバインディングをクレートのトップレベルから直接参照できるように再エクスポートする
pub use memory::{InMemoryInputBinding, InMemoryOutputBinding};
// バインディングトレイトおよびデータ型をトップレベルから直接参照できるように再エクスポートする
pub use traits::{BindingData, BindingResponse, InputBinding, OutputBinding};
