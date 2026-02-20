//! k1s0-serviceauth: サービス間認証ライブラリ。
//!
//! OAuth2 Client Credentials フローによるサービストークン管理を提供する。
//! Istio の mTLS と SPIFFE ID によるワークロードアイデンティティ検証もサポートする。
//!
//! # 使い方
//!
//! ```ignore
//! use k1s0_serviceauth::{HttpServiceAuthClient, ServiceAuthClient, ServiceAuthConfig};
//!
//! let config = ServiceAuthConfig::new(
//!     "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/token",
//!     "my-service",
//!     "my-secret",
//! )
//! .with_jwks_uri("https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs");
//!
//! let client = HttpServiceAuthClient::new(config).unwrap();
//!
//! // キャッシュ付きトークン取得（期限前に自動リフレッシュ）
//! let bearer = client.get_cached_token().await.unwrap();
//!
//! // SPIFFE ID 検証
//! let spiffe = client
//!     .validate_spiffe_id("spiffe://k1s0.internal/ns/system/sa/auth-service", "system")
//!     .unwrap();
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod token;

pub use client::{HttpServiceAuthClient, ServiceAuthClient, ServiceClaims};
pub use config::ServiceAuthConfig;
pub use error::ServiceAuthError;
pub use token::{ServiceToken, SpiffeId};

#[cfg(feature = "mock")]
pub use client::MockServiceAuthClient;
