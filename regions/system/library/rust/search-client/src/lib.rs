pub mod client;
pub mod document;
pub mod error;
pub mod query;

pub use client::SearchClient;
pub use document::{BulkFailure, BulkResult, FieldMapping, IndexDocument, IndexMapping, IndexResult};
pub use error::SearchError;
pub use query::{FacetBucket, Filter, SearchQuery, SearchResult};

#[cfg(feature = "mock")]
pub use client::MockSearchClient;
