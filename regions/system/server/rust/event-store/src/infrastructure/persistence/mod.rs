// PostgreSQL 永続化実装群。
// usecase 層はこれらの具体型に直接依存せず、domain トレイトを介してのみ操作する。

pub mod event_postgres;
pub mod snapshot_postgres;
pub mod stream_postgres;
pub mod transactional_append;

pub use event_postgres::EventPostgresRepository;
pub use snapshot_postgres::SnapshotPostgresRepository;
pub use stream_postgres::StreamPostgresRepository;
pub use transactional_append::TransactionalAppendAdapter;
