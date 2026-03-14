// AIクライアントで使用する型定義
// リクエスト・レスポンス・エラーの各構造体を定義する

use serde::{Deserialize, Serialize};

// チャットメッセージを表す構造体（ロールとコンテンツを保持）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// メッセージの役割（system, user, assistant など）
    pub role: String,
    /// メッセージの内容
    pub content: String,
}

// チャット補完リクエストの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteRequest {
    /// 使用するモデルID
    pub model: String,
    /// 会話メッセージの一覧
    pub messages: Vec<ChatMessage>,
    /// 生成する最大トークン数（省略可能）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    /// サンプリング温度（省略可能）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// ストリーミングモードを使用するかどうか
    pub stream: bool,
}

// チャット補完レスポンスの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteResponse {
    /// レスポンスの一意識別子
    pub id: String,
    /// 使用されたモデルID
    pub model: String,
    /// 生成されたテキスト内容
    pub content: String,
    /// 入力に使用されたトークン数
    pub prompt_tokens: i32,
    /// 生成に使用されたトークン数
    pub completion_tokens: i32,
}

// 埋め込みリクエストの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// 使用する埋め込みモデルID
    pub model: String,
    /// 埋め込み対象のテキスト一覧
    pub inputs: Vec<String>,
}

// 埋め込みレスポンスの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    /// 使用されたモデルID
    pub model: String,
    /// 各入力に対応する埋め込みベクトルの一覧
    pub embeddings: Vec<Vec<f32>>,
}

// AIモデル情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    /// モデルの一意識別子
    pub id: String,
    /// モデルの表示名
    pub name: String,
    /// プロバイダー名（openai, anthropic など）
    pub provider: String,
    /// コンテキストウィンドウサイズ（トークン数）
    pub context_window: i32,
    /// モデルが有効かどうか
    pub enabled: bool,
}

// AIクライアントのエラー型（thiserrorで実装）
#[derive(Debug, thiserror::Error)]
pub enum AiClientError {
    /// HTTP通信エラー
    #[error("HTTP error: {0}")]
    HttpError(String),
    /// JSONシリアライズ/デシリアライズエラー
    #[error("JSON error: {0}")]
    JsonError(String),
    /// サービス利用不可エラー
    #[error("Service unavailable: {0}")]
    Unavailable(String),
}
