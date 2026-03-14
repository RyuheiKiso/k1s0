pub mod client;
pub mod config;
pub mod error;
pub mod message;
pub mod state;
// native フィーチャーが有効な場合のみ tokio-tungstenite 実装を公開する
#[cfg(feature = "native")]
pub mod native_client;

#[cfg(feature = "mock")]
pub use client::MockWsClient;
pub use client::{InMemoryWsClient, WsClient};
pub use config::WsConfig;
pub use error::WsError;
pub use message::{CloseFrame, WsMessage};
pub use state::ConnectionState;
#[cfg(feature = "native")]
pub use native_client::TungsteniteWsClient;
