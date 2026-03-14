// AIクライアントで使用する型定義
// Go の types.go と対応するリクエスト・レスポンス・エラーの各構造体を定義する

use serde::{Deserialize, Serialize};

// チャットメッセージを表す構造体（ロールとコンテンツを保持）
// role は "user", "assistant", "system" のいずれか
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// メッセージの役割
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
    /// ストリーミングモードを使用するかどうか（省略可能）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

// トークン使用量を保持する構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// 入力トークン数
    pub input_tokens: i32,
    /// 出力トークン数
    pub output_tokens: i32,
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
    /// トークン使用量
    pub usage: Usage,
}

// 埋め込みリクエストの構造体
// Go の EmbedRequest.texts に対応する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// 使用する埋め込みモデルID
    pub model: String,
    /// 埋め込み対象のテキスト一覧
    pub texts: Vec<String>,
}

// 埋め込みレスポンスの構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    /// 使用されたモデルID
    pub model: String,
    /// 各入力に対応する埋め込みベクトルの一覧
    pub embeddings: Vec<Vec<f32>>,
}

// モデルの基本情報を表す構造体
// Go の ModelInfo と対応する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// モデルの一意識別子
    pub id: String,
    /// モデルの表示名
    pub name: String,
    /// モデルの説明
    pub description: String,
}

// AIクライアントのエラー型
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
