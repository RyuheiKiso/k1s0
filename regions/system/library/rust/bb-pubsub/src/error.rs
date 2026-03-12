use thiserror::Error;

#[derive(Debug, Error)]
pub enum PubSubError {
    #[error("パブリッシュエラー: {0}")]
    Publish(String),
    #[error("サブスクライブエラー: {0}")]
    Subscribe(String),
    #[error("サブスクリプションが見つかりません: {0}")]
    SubscriptionNotFound(String),
    #[error("接続エラー: {0}")]
    Connection(String),
    #[error("シリアライズエラー: {0}")]
    Serialization(String),
    #[error("コンポーネントエラー: {0}")]
    Component(#[from] k1s0_building_blocks::ComponentError),
}
