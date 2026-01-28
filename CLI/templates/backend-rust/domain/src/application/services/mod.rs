//! アプリケーションサービス定義
//!
//! このモジュールにはアプリケーションサービス（ユースケースの実装）を定義します。
//! アプリケーションサービスはドメインオブジェクトを調整してビジネスプロセスを実行します。
//!
//! # 特徴
//!
//! - ユースケースごとに1つのサービスを定義
//! - ドメインオブジェクトの操作を調整
//! - トランザクション境界を管理
//! - 結果を DTO として返す
//!
//! # 例
//!
//! ```rust,ignore
//! pub struct CreateWorkOrderService<R> {
//!     repository: R,
//! }
//!
//! impl<R: WorkOrderRepository> CreateWorkOrderService<R> {
//!     pub async fn execute(&self, input: CreateWorkOrderInput) -> Result<WorkOrderDto, AppError> {
//!         // 1. ドメインオブジェクトを生成
//!         // 2. ビジネスルールを適用
//!         // 3. 永続化
//!         // 4. 結果を返す
//!     }
//! }
//! ```

// TODO: アプリケーションサービスをここに追加
// 例:
// mod create_work_order;
// pub use create_work_order::CreateWorkOrderService;
