// ドメインリポジトリトレイト群。
// usecase 層はこれらのトレイトにのみ依存し、infrastructure 具体型には依存しない。

pub mod event_repository;
pub mod transactional_port;

pub use event_repository::EventRepository;
pub use event_repository::EventStreamRepository;
pub use event_repository::SnapshotRepository;
pub use transactional_port::TransactionalAppendPort;
