//! 値オブジェクト定義
//!
//! このモジュールにはドメインの値オブジェクトを定義します。
//! 値オブジェクトは不変で、値の等価性によって比較されます。
//!
//! # 特徴
//!
//! - 不変（イミュータブル）
//! - 識別子を持たない
//! - 等価性は全ての属性で判断される
//! - 自己完結した検証ロジックを持つ
//!
//! # 例
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! pub struct LotNumber(String);
//!
//! impl LotNumber {
//!     pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
//!         let value = value.into();
//!         // バリデーションロジック
//!         Ok(Self(value))
//!     }
//! }
//! ```

// TODO: 値オブジェクトをここに追加
// 例:
// mod lot_number;
// pub use lot_number::LotNumber;
