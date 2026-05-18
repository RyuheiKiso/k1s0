// tier2 OAuth Token Exchange モジュール（設計方針 16 / 外部システム統合）
// RFC 8693 OAuth 2.0 Token Exchange を使用してサービス間トークン変換を行う
// authorization server に HTTP POST して access_token を取得する

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: トークン交換レスポンスのデシリアライズに使用する
use serde::{Deserialize, Serialize};
// reqwest: HTTP クライアント（Token Exchange HTTP POST に使用する）
use reqwest::Client;
// Duration: HTTP タイムアウト設定に使用する
use std::time::Duration;
// tracing マクロのために tracing クレートをインポートする
use tracing;

// ExchangeError: Token Exchange 中に発生するエラーの列挙型
#[derive(Debug)]
pub enum ExchangeError {
    // HTTP 送信エラー: ネットワーク障害やタイムアウト時に発生する
    Http(reqwest::Error),
    // HTTP ステータスエラー: 2xx 以外のレスポンスコードを受信した場合に発生する
    HttpStatus(u16),
    // レスポンスパースエラー: JSON デシリアライズに失敗した場合に発生する
    Parse(reqwest::Error),
    // access_token フィールドが欠落している場合に発生する
    MissingToken,
}

// impl Display for ExchangeError: エラーメッセージを文字列化する
impl std::fmt::Display for ExchangeError {
    // fmt: エラーメッセージを文字列化して返す
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // エラー種別ごとにメッセージを返す
        match self {
            // HTTP 送信エラーメッセージを返す
            ExchangeError::Http(e) => write!(f, "HTTP 送信エラー: {}", e),
            // HTTP ステータスエラーメッセージを返す
            ExchangeError::HttpStatus(code) => write!(f, "HTTP ステータスエラー: {}", code),
            // レスポンスパースエラーメッセージを返す
            ExchangeError::Parse(e) => write!(f, "レスポンスパースエラー: {}", e),
            // access_token 欠落エラーメッセージを返す
            ExchangeError::MissingToken => write!(f, "access_token フィールドが欠落している"),
        }
    }
}

// TokenExchangeResponse: Token Exchange レスポンスの構造体（RFC 8693 準拠）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExchangeResponse {
    // 発行されたアクセストークン
    pub access_token: String,
    // トークン種別（常に "Bearer"）
    pub token_type: String,
    // トークン有効期間（秒単位 / wall clock TTL 計算禁止のため HLC で管理する）
    pub expires_in: Option<u64>,
    // 付与されたスコープ（要求と異なる場合あり）
    pub scope: Option<String>,
}

// OAuthTokenExchange: RFC 8693 Token Exchange の実装
// authorization server に HTTP POST して access_token を取得する
pub struct OAuthTokenExchange {
    // reqwest HTTP クライアント（コネクションプールを共有する）
    client: Client,
    // Token Exchange エンドポイント URL（Keycloak の token エンドポイント）
    token_endpoint: String,
    // クライアント ID（サービスアカウントの識別子）
    client_id: String,
    // クライアントシークレット（OpenBao Transit 経由で管理する）
    client_secret: String,
    // 対象オーディエンス（target_audience として Token Exchange リクエストに含める）
    target_audience: String,
}

impl OAuthTokenExchange {
    // OAuthTokenExchange を生成する
    // token_endpoint: Keycloak の token エンドポイント URL
    // client_id: サービスアカウントのクライアント ID
    // client_secret: サービスアカウントのクライアントシークレット
    // target_audience: トークンの対象オーディエンス
    pub fn new(
        token_endpoint: String,
        client_id: String,
        client_secret: String,
        target_audience: String,
    ) -> Result<Self, ExchangeError> {
        // reqwest Client をタイムアウト設定で生成する
        let client = Client::builder()
            // 接続タイムアウトを 10 秒に設定する
            .connect_timeout(Duration::from_secs(10))
            // リクエストタイムアウトを 30 秒に設定する
            .timeout(Duration::from_secs(30))
            // クライアントをビルドする
            .build()
            .map_err(ExchangeError::Http)?;
        // 設定を保持するインスタンスを生成する
        Ok(Self {
            client,
            token_endpoint,
            client_id,
            client_secret,
            target_audience,
        })
    }

    // RFC 8693 Token Exchange: authorization server に POST する
    // subject_token: 変換元トークン（ユーザーの Bearer トークン等）
    // 変換されたアクセストークン文字列を返す
    pub async fn exchange(&self, subject_token: &str) -> Result<String, ExchangeError> {
        // Token Exchange リクエストボディを構築する
        let params = [
            // grant_type: RFC 8693 Token Exchange の固定値
            ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
            // subject_token: 変換元トークン
            ("subject_token", subject_token),
            // subject_token_type: Bearer トークン種別を指定する
            ("subject_token_type", "urn:ietf:params:oauth:token-type:access_token"),
            // audience: 対象サービス識別子
            ("audience", &self.target_audience),
            // client_id: サービスアカウント識別子
            ("client_id", &self.client_id),
            // client_secret: サービスアカウントのシークレット (form パラメータで送信する)
            ("client_secret", &self.client_secret),
        ];
        // authorization server に HTTP POST を送信する
        tracing::debug!(
            endpoint = %self.token_endpoint,
            audience = %self.target_audience,
            "Token Exchange リクエストを送信する"
        );
        // HTTP POST リクエストを form エンコードで送信する
        let response = self.client
            // token_endpoint に POST リクエストを送信する
            .post(&self.token_endpoint)
            // form エンコードされたパラメータをボディに設定する
            .form(&params)
            // HTTP リクエストを送信する
            .send()
            .await
            .map_err(ExchangeError::Http)?;
        // 2xx 以外はエラーとして扱う
        if !response.status().is_success() {
            // レスポンスのステータスコードを取得する
            let status = response.status().as_u16();
            // ステータスエラーを返す
            return Err(ExchangeError::HttpStatus(status));
        }
        // レスポンスから access_token を抽出する
        let body: serde_json::Value = response.json().await.map_err(ExchangeError::Parse)?;
        // access_token フィールドを文字列として取得する
        body["access_token"].as_str()
            // String に変換する
            .map(String::from)
            // access_token が欠落している場合はエラーを返す
            .ok_or(ExchangeError::MissingToken)
    }

    // subject_token を指定して TokenExchangeResponse 全体を取得する (詳細情報が必要な場合に使用する)
    // subject_token: 変換元トークン
    // TokenExchangeResponse 構造体全体を返す
    pub async fn exchange_full(
        &self,
        subject_token: &str,
    ) -> Result<TokenExchangeResponse, ExchangeError> {
        // Token Exchange リクエストボディを構築する (exchange と同一)
        let params = [
            // grant_type: RFC 8693 Token Exchange の固定値
            ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
            // subject_token: 変換元トークン
            ("subject_token", subject_token),
            // subject_token_type: Bearer トークン種別を指定する
            ("subject_token_type", "urn:ietf:params:oauth:token-type:access_token"),
            // audience: 対象サービス識別子
            ("audience", &self.target_audience),
            // client_id: サービスアカウント識別子
            ("client_id", &self.client_id),
            // client_secret: サービスアカウントのシークレット
            ("client_secret", &self.client_secret),
        ];
        // HTTP POST リクエストを form エンコードで送信する
        let response = self.client
            // token_endpoint に POST リクエストを送信する
            .post(&self.token_endpoint)
            // form エンコードされたパラメータをボディに設定する
            .form(&params)
            // HTTP リクエストを送信する
            .send()
            .await
            .map_err(ExchangeError::Http)?;
        // 2xx 以外はエラーとして扱う
        if !response.status().is_success() {
            // レスポンスのステータスコードを取得する
            let status = response.status().as_u16();
            // ステータスエラーを返す
            return Err(ExchangeError::HttpStatus(status));
        }
        // レスポンスを TokenExchangeResponse にデシリアライズして返す
        response.json::<TokenExchangeResponse>().await.map_err(ExchangeError::Parse)
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    // OAuthTokenExchange の生成が正常に動作することを確認する
    #[test]
    fn test_oauth_token_exchange_new() {
        // テスト用の設定で OAuthTokenExchange を生成する
        let result = OAuthTokenExchange::new(
            "https://keycloak.example.com/token".to_string(),
            "tier2-service".to_string(),
            "test-secret".to_string(),
            "https://api.example.com".to_string(),
        );
        // 生成が成功することを確認する
        assert!(result.is_ok());
    }

    // ExchangeError::MissingToken の Display が正しいことを確認する
    #[test]
    fn test_exchange_error_missing_token_display() {
        // MissingToken エラーを生成する
        let err = ExchangeError::MissingToken;
        // Display 実装が空文字列でないことを確認する
        assert!(!err.to_string().is_empty());
    }

    // ExchangeError::HttpStatus の Display が正しいことを確認する
    #[test]
    fn test_exchange_error_http_status_display() {
        // HttpStatus 401 エラーを生成する
        let err = ExchangeError::HttpStatus(401);
        // Display 実装に "401" が含まれることを確認する
        assert!(err.to_string().contains("401"));
    }
}
