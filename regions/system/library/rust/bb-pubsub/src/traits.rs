use std::collections::HashMap;

use async_trait::async_trait;

use crate::PubSubError;

/// PubSub メッセージ。
#[derive(Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub id: String,
}

/// メッセージハンドラーのトレイト。
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, message: Message) -> Result<(), PubSubError>;
}

/// PubSub はパブリッシュ・サブスクライブ機能の抽象インターフェース。
/// Component トレイトを拡張する。
#[async_trait]
pub trait PubSub: k1s0_bb_core::Component {
    /// トピックにメッセージをパブリッシュする。
    async fn publish(
        &self,
        topic: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), PubSubError>;

    /// トピックをサブスクライブし、サブスクリプション ID を返す。
    async fn subscribe(
        &self,
        topic: &str,
        handler: Box<dyn MessageHandler>,
    ) -> Result<String, PubSubError>;

    /// サブスクリプションを解除する。
    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), PubSubError>;
}
