pub mod search_opensearch;
pub mod search_postgres;

pub use search_opensearch::SearchOpenSearchRepository;
pub use search_postgres::SearchPostgresRepository;
