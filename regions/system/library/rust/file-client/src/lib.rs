pub mod client;
pub mod config;
pub mod error;
pub mod model;
#[cfg(feature = "direct-mode")]
pub mod multipart;
#[cfg(feature = "direct-mode")]
pub mod s3;

pub use client::{FileClient, InMemoryFileClient, ServerFileClient};
pub use config::FileClientConfig;
pub use error::FileClientError;
pub use model::{FileMetadata, PresignedUrl};
#[cfg(feature = "direct-mode")]
pub use multipart::{MultipartPart, MultipartUploadSession};
#[cfg(feature = "direct-mode")]
pub use s3::S3FileClient;

#[cfg(feature = "mock")]
pub use client::MockFileClient;
