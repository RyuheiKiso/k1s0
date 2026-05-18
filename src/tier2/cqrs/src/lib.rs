// k1s0-tier2-cqrs クレートのルートモジュール（設計方針 15 / CQRS 読み取りモデル）
// ReadModelProjector トレイトと PgVectorQuery を公開 API として提供する

// ReadModelProjector トレイトおよびドメインイベント定義モジュール
pub mod projector;
// pgvector 類似検索クエリモジュール
pub mod pgvector_query;

// 公開型の再エクスポート（tier2 CQRS 公開 API 表面を最小化する）
pub use projector::{DomainEvent, ReadModelProjector, ReadModelRegistry};
// PgVectorQuery と VectorSearchResult を公開する
pub use pgvector_query::{PgVectorQuery, VectorSearchResult};
