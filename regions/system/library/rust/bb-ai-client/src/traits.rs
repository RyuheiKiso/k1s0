// AIクライアントのトレイト定義
// AIバックエンドとの通信インターフェースを抽象化する

use crate::types::{
    AiClientError, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse, ModelInfo,
};

// AiClient トレイト: AIバックエンドとの通信インターフェース
// mock featureが有効な場合、mockallによるモック自動生成を行う
#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait::async_trait]
pub trait AiClient: Send + Sync {
    /// チャット補完リクエストを送信し、レスポンスを返す
    async fn complete(&self, req: &CompleteRequest) -> Result<CompleteResponse, AiClientError>;

    /// テキストの埋め込みベクトルを取得する
    async fn embed(&self, req: &EmbedRequest) -> Result<EmbedResponse, AiClientError>;

    /// 利用可能なモデルの一覧を取得する
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AiClientError>;
}
