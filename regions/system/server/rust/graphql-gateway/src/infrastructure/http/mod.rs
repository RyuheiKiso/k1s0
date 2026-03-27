// HTTP クライアントモジュール。
// gRPC を提供しないサービス（service-catalog 等）に対して
// reqwest ベースの REST クライアントを提供する。

pub mod service_catalog_client;

pub use service_catalog_client::ServiceCatalogHttpClient;
