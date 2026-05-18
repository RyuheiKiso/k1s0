// tier2 Webhook 配信モジュール（設計方針 16 / 外部システム統合）
// HMAC-SHA256 署名付き Webhook を外部エンドポイントに HTTP POST で配信する

// anyhow: Result 型に使用する
use anyhow::Result;
// hmac: HMAC-SHA256 署名計算に使用する
use hmac::{Hmac, Mac};
// sha2: SHA-256 ハッシュ関数に使用する
use sha2::Sha256;
// hex: 署名バイト列を小文字 hex 文字列に変換する
use hex::encode as hex_encode;
// reqwest: HTTP クライアント（HTTP POST 送信に使用する）
use reqwest::Client;
// Duration: HTTP タイムアウト設定に使用する
use std::time::Duration;
// tracing マクロのために tracing クレートをインポートする
use tracing;

// HmacSha256 型エイリアス（繰り返しを避けるために定義する）
type HmacSha256 = Hmac<Sha256>;

// WebhookError: Webhook 配信中に発生するエラーの列挙型
#[derive(Debug)]
pub enum WebhookError {
    // HMAC 署名計算エラー: シークレットキーが不正な場合に発生する
    SignatureCompute(String),
    // HTTP 送信エラー: ネットワーク障害やタイムアウト時に発生する
    Http(reqwest::Error),
    // HTTP ステータスエラー: 2xx 以外のレスポンスコードを受信した場合に発生する
    HttpStatus(u16),
    // シリアライズエラー: ペイロードの JSON 変換に失敗した場合に発生する
    Serialization(serde_json::Error),
}

// impl Display for WebhookError: エラーメッセージを文字列化する
impl std::fmt::Display for WebhookError {
    // fmt: エラーメッセージを文字列化して返す
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // エラー種別ごとにメッセージを返す
        match self {
            // HMAC 署名計算エラーメッセージを返す
            WebhookError::SignatureCompute(msg) => write!(f, "HMAC 署名計算エラー: {}", msg),
            // HTTP 送信エラーメッセージを返す
            WebhookError::Http(e) => write!(f, "HTTP 送信エラー: {}", e),
            // HTTP ステータスエラーメッセージを返す
            WebhookError::HttpStatus(code) => write!(f, "HTTP ステータスエラー: {}", code),
            // シリアライズエラーメッセージを返す
            WebhookError::Serialization(e) => write!(f, "シリアライズエラー: {}", e),
        }
    }
}

// WebhookPayload: Webhook で配信するペイロードの構造体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookPayload {
    // イベント種別 (domain_event の event_type に対応する)
    pub event_type: String,
    // イベント本体 (domain_event の payload に対応する)
    pub body: serde_json::Value,
}

// WebhookDelivery: HMAC-SHA256 署名付き Webhook 配信の実装
// HTTP POST で JSON ペイロードと X-K1s0-Signature ヘッダを送信する
pub struct WebhookDelivery {
    // reqwest HTTP クライアント（コネクションプールを共有する）
    client: Client,
    // Webhook 配信先エンドポイント URL
    endpoint_url: String,
    // HMAC 署名用シークレット（OpenBao Transit 経由で管理する）
    secret: String,
    // 最大リトライ回数（指数バックオフで再試行する）
    max_retries: u32,
}

impl WebhookDelivery {
    // WebhookDelivery を生成する
    // endpoint_url: Webhook 配信先エンドポイント URL
    // secret: HMAC-SHA256 署名用シークレット
    // max_retries: 最大リトライ回数
    pub fn new(endpoint_url: String, secret: String, max_retries: u32) -> Result<Self, WebhookError> {
        // reqwest Client をタイムアウト設定で生成する
        let client = Client::builder()
            // 接続タイムアウトを 10 秒に設定する
            .connect_timeout(Duration::from_secs(10))
            // リクエストタイムアウトを 30 秒に設定する
            .timeout(Duration::from_secs(30))
            // クライアントをビルドする
            .build()
            .map_err(WebhookError::Http)?;
        // 設定を保持するインスタンスを生成する
        Ok(Self {
            client,
            endpoint_url,
            secret,
            max_retries,
        })
    }

    // HMAC-SHA256 署名を計算して hex 文字列として返す
    // secret: Webhook 署名シークレット（OpenBao Transit 経由で管理する）
    // body: 署名対象の JSON ペイロードバイト列
    pub fn compute_signature(secret: &str, body: &[u8]) -> Result<String, WebhookError> {
        // HMAC-SHA256 インスタンスをシークレットキーで初期化する
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| WebhookError::SignatureCompute(e.to_string()))?;
        // ペイロードバイト列を HMAC に入力する
        mac.update(body);
        // HMAC の計算結果バイト列を取得する
        let result = mac.finalize();
        // バイト列を小文字 hex 文字列に変換して返す
        Ok(format!("sha256={}", hex_encode(result.into_bytes())))
    }

    // HTTP POST で webhook を配送する: reqwest Client を使用
    // payload: 配信する Webhook ペイロード構造体
    pub async fn deliver(&self, payload: &WebhookPayload) -> Result<(), WebhookError> {
        // ペイロードを JSON バイト列にシリアライズする
        let body_bytes = serde_json::to_vec(payload)
            .map_err(WebhookError::Serialization)?;
        // HMAC-SHA256 署名を計算する
        let signature = Self::compute_signature(&self.secret, &body_bytes)?;
        // リトライカウンターを初期化する
        let mut attempt = 0u32;
        // 最大リトライ回数まで送信を試みる
        loop {
            // HTTP POST リクエストを送信する (タイムアウト 30 秒)
            let response = self.client
                // エンドポイント URL に POST リクエストを送信する
                .post(&self.endpoint_url)
                // HMAC-SHA256 署名をヘッダに設定する
                .header("X-K1s0-Signature", &signature)
                // JSON ペイロードをボディに設定する
                .json(payload)
                // HTTP リクエストを送信する
                .send()
                .await
                .map_err(WebhookError::Http)?;
            // 2xx 以外はエラーとして扱う
            if !response.status().is_success() {
                // レスポンスのステータスコードを取得する
                let status = response.status().as_u16();
                // リトライ可能な場合 (5xx) かつ最大リトライ回数未満の場合は再試行する
                if status >= 500 && attempt < self.max_retries {
                    // 指数バックオフで待機する (1s, 2s, 4s, ...)
                    let wait = Duration::from_secs(1u64 << attempt);
                    // リトライ前に待機する旨をログに記録する
                    tracing::warn!(
                        attempt = attempt,
                        status = status,
                        wait_secs = wait.as_secs(),
                        "Webhook 配信失敗: リトライする"
                    );
                    // 待機する
                    tokio::time::sleep(wait).await;
                    // リトライカウンターをインクリメントする
                    attempt += 1;
                    // ループの先頭に戻る
                    continue;
                }
                // リトライ不可または最大リトライ回数超過の場合はエラーを返す
                return Err(WebhookError::HttpStatus(status));
            }
            // 配信成功をログに記録する
            tracing::debug!(
                url = %self.endpoint_url,
                attempt = attempt,
                "Webhook 配信成功"
            );
            // 配信成功を返す
            return Ok(());
        }
    }

    // 後方互換のために旧 deliver シグネチャも残す（同期ラッパー不要: 非推奨として保持）
    // url: 配信先 URL (self.endpoint_url を使う場合は deliver を使う)
    // payload: 配信する JSON ペイロード
    // secret: HMAC 署名用シークレット
    pub fn compute_and_log(
        &self,
        url: &str,
        payload: &serde_json::Value,
        secret: &str,
    ) -> anyhow::Result<()> {
        // ペイロードを JSON バイト列にシリアライズする
        let body = serde_json::to_vec(payload)?;
        // HMAC-SHA256 署名を計算する
        let signature = Self::compute_signature(secret, &body)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        // 署名をログに記録する（実際の HTTP 送信は deliver() で行う）
        tracing::debug!(
            url = url,
            signature = %signature,
            max_retries = self.max_retries,
            "Webhook 署名計算完了 (deliver() を使用して送信する)"
        );
        // 計算成功
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    // compute_signature が期待する形式の署名を返すことを確認する
    #[test]
    fn test_compute_signature_format() {
        // テスト用シークレットとペイロードを定義する
        let secret = "test-secret-key";
        let body = b"test-payload";
        // HMAC-SHA256 署名を計算する
        let sig = WebhookDelivery::compute_signature(secret, body).unwrap();
        // 署名が "sha256=" で始まることを確認する
        assert!(sig.starts_with("sha256="));
        // 署名が小文字 hex 文字列であることを確認する (7 + 64 = 71 文字)
        assert_eq!(sig.len(), 7 + 64);
    }

    // compute_signature が同一入力に対して同一署名を返すことを確認する
    #[test]
    fn test_compute_signature_deterministic() {
        // テスト用シークレットとペイロードを定義する
        let secret = "k1s0-hmac-secret";
        let body = b"webhook-body";
        // 2 回計算して同一結果であることを確認する
        let sig1 = WebhookDelivery::compute_signature(secret, body).unwrap();
        let sig2 = WebhookDelivery::compute_signature(secret, body).unwrap();
        // 決定的な署名計算であることを確認する
        assert_eq!(sig1, sig2);
    }
}
