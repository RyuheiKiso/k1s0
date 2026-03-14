// AIクライアントライブラリのエントリポイント
// AI バックエンド（チャット補完、埋め込み、モデル一覧）との通信機能を提供する

pub mod client;
pub mod memory;
pub mod traits;
pub mod types;

// 主要な型とトレイトを再エクスポートする
pub use client::HttpAiClient;
pub use memory::InMemoryAiClient;
pub use traits::AiClient;
pub use types::{
    AiClientError, ChatMessage, CompleteRequest, CompleteResponse,
    EmbedRequest, EmbedResponse, ModelInfo, Usage,
};
