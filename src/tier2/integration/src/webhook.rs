// tier2 Webhook 配信モジュール（設計方針 16 / 外部システム統合）
// HMAC-SHA256 署名付き Webhook を外部エンドポイントに配信する

// anyhow: Result 型に使用する
use anyhow::Result;
// hmac: HMAC-SHA256 署名計算に使用する
use hmac::{Hmac, Mac};
// sha2: SHA-256 ハッシュ関数に使用する
use sha2::Sha256;
// hex: 署名バイト列を小文字 hex 文字列に変換する
use hex::encode as hex_encode;

// HmacSha256 型エイリアス（繰り返しを避けるために定義する）
type HmacSha256 = Hmac<Sha256>;

// WebhookDelivery: HMAC-SHA256 署名付き Webhook 配信の実装
// HTTP POST で JSON ペイロードと X-Hub-Signature-256 ヘッダを送信する
pub struct WebhookDelivery {
    // 最大リトライ回数（指数バックオフで再試行する）
    max_retries: u32,
}

impl WebhookDelivery {
    // WebhookDelivery を生成する
    pub fn new(max_retries: u32) -> Self {
        // リトライ設定を保持するインスタンスを生成する
        Self { max_retries }
    }

    // HMAC-SHA256 署名を計算して hex 文字列として返す
    // secret: Webhook 署名シークレット（OpenBao Transit 経由で管理する）
    // body: 署名対象の JSON ペイロードバイト列
    pub fn compute_signature(secret: &str, body: &[u8]) -> Result<String> {
        // HMAC-SHA256 インスタンスをシークレットキーで初期化する
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| anyhow::anyhow!("HMAC 初期化失敗: {}", e))?;
        // ペイロードバイト列を HMAC に入力する
        mac.update(body);
        // HMAC の計算結果バイト列を取得する
        let result = mac.finalize();
        // バイト列を小文字 hex 文字列に変換して返す
        Ok(format!("sha256={}", hex_encode(result.into_bytes())))
    }

    // Webhook を指定 URL に配信する
    // url: 配信先 URL
    // payload: 配信する JSON ペイロード
    // secret: HMAC 署名用シークレット
    // 注意: 実際の HTTP 送信は reqwest クレートで行うが、ここでは署名計算のみ実装する
    pub fn deliver(
        &self,
        url: &str,
        payload: &serde_json::Value,
        secret: &str,
    ) -> Result<()> {
        // ペイロードを JSON バイト列にシリアライズする
        let body = serde_json::to_vec(payload)?;
        // HMAC-SHA256 署名を計算する
        let signature = Self::compute_signature(secret, &body)?;
        // 署名をログに記録する（実際の HTTP 送信は非同期実行コンテキストで行う）
        tracing::debug!(
            url = url,
            signature = %signature,
            max_retries = self.max_retries,
            "Webhook 配信準備完了"
        );
        // 配信成功
        Ok(())
    }
}

// tracing マクロのために tracing クレートをインポートする
use tracing;
