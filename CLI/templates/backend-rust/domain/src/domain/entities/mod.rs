//! エンティティ定義
//!
//! このモジュールにはドメインのエンティティを定義します。
//! エンティティは一意の識別子を持ち、ライフサイクルを通じて同一性を保ちます。
//!
//! # 特徴
//!
//! - 一意の識別子（ID）を持つ
//! - 状態が変化しても同一のエンティティとして扱われる
//! - 等価性は識別子で判断される
//!
//! # 例
//!
//! ```rust,ignore
//! pub struct Order {
//!     id: OrderId,
//!     customer_id: CustomerId,
//!     items: Vec<OrderItem>,
//!     status: OrderStatus,
//! }
//! ```

// TODO: エンティティをここに追加
// 例:
// mod bom;
// pub use bom::Bom;
