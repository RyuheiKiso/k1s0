//! ドメインサービス定義
//!
//! このモジュールにはドメインサービスを定義します。
//! ドメインサービスは、単一のエンティティに属さないビジネスロジックを実装します。
//!
//! # 用途
//!
//! - 複数のエンティティにまたがる操作
//! - 外部サービスとの連携が必要なビジネスルール
//! - 計算や変換のロジック
//!
//! # 例
//!
//! ```rust,ignore
//! pub struct CapacityCalculator;
//!
//! impl CapacityCalculator {
//!     pub fn calculate_available_capacity(
//!         &self,
//!         production_line: &ProductionLine,
//!         schedule: &ProductionSchedule,
//!     ) -> Capacity {
//!         // ビジネスロジック
//!     }
//! }
//! ```

// TODO: ドメインサービスをここに追加
// 例:
// mod capacity_calculator;
// pub use capacity_calculator::CapacityCalculator;
