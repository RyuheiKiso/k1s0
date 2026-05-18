// openbao.rs — k1s0 tier1 BFL: OpenBao Transit API クライアント
// 05_鍵管理適合仕様.md §5 層 defense-in-depth 層 B「runtime: OpenBao Transit 委譲」を実装する。
// KeyHandle.sign / KeyHandle.verify が呼び出す実際の署名・検証バックエンド。
// BFL がホストする単一インスタンスを 4 言語 Library から HTTP 経由で利用する設計。

// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::{Context, Result};
// base64: OpenBao Transit API の input / signature フィールドの Base64 エンコード/デコード
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STD};
// reqwest: OpenBao Transit API への HTTP クライアント
use reqwest::Client;
// serde_json: OpenBao API レスポンスの JSON パース
use serde_json::Value;
// tracing: OpenBao API 呼び出しのトレーシング
use tracing::{debug, instrument};

/// OpenBaoTransitClient は OpenBao Transit API へのクライアントを宣言する。
/// sign / verify エンドポイントを HTTP で呼び出し、key bytes を tier1 境界に露出しない。
pub struct OpenBaoTransitClient {
    // base_url: OpenBao サーバーのベース URL（例: http://openbao.k1s0.svc:8200）
    base_url: String,
    // token: OpenBao Vault トークン（X-Vault-Token ヘッダーで送信する）
    token: String,
    // client: reqwest 非同期 HTTP クライアント（接続プールを再利用する）
    client: Client,
}

impl OpenBaoTransitClient {
    /// new は OpenBaoTransitClient を構築する。
    /// base_url と token は環境変数 OPENBAO_ADDR / OPENBAO_TOKEN から取得する。
    pub fn new_from_env() -> Result<Self> {
        // OPENBAO_ADDR 環境変数からベース URL を取得する（デフォルト: http://openbao.k1s0.svc:8200）
        let base_url = std::env::var("OPENBAO_ADDR")
            .unwrap_or_else(|_| "http://openbao.k1s0.svc:8200".to_string());
        // OPENBAO_TOKEN 環境変数からトークンを取得する（未設定時はエラー）
        let token = std::env::var("OPENBAO_TOKEN")
            .context("OPENBAO_TOKEN 環境変数が設定されていない")?;
        // reqwest Client を構築する（接続タイムアウト 5 秒）
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .context("reqwest Client の構築に失敗した")?;
        // OpenBaoTransitClient を返す
        Ok(Self { base_url, token, client })
    }

    /// sign は payload を OpenBao Transit の key_name で署名して署名バイト列を返す。
    /// POST /v1/transit/sign/{key_name} を呼び出す。
    #[instrument(skip(self, payload), fields(key_name = %key_name))]
    pub async fn sign(&self, key_name: &str, payload: &[u8]) -> Result<Vec<u8>> {
        // payload を Base64 エンコードする（OpenBao Transit の input フィールドは Base64 要求）
        let input_b64 = BASE64_STD.encode(payload);
        // sign エンドポイント URL を構築する
        let url = format!("{}/v1/transit/sign/{}", self.base_url, key_name);
        debug!(url = %url, "OpenBao Transit sign 呼び出し");
        // POST リクエストを送信する
        let resp = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&serde_json::json!({ "input": input_b64 }))
            .send()
            .await
            .context("OpenBao Transit sign リクエスト送信に失敗した")?;
        // HTTP ステータスを確認する
        if !resp.status().is_success() {
            // エラーステータス時はボディを取得してエラーメッセージに含める
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenBao Transit sign 失敗: HTTP {} body={}", status, body));
        }
        // レスポンス JSON をパースする
        let data: Value = resp.json().await
            .context("OpenBao Transit sign レスポンスの JSON パースに失敗した")?;
        // signature フィールドを取得する（"vault:v1:<base64>" 形式）
        let sig_str = data["data"]["signature"].as_str()
            .ok_or_else(|| anyhow::anyhow!("OpenBao Transit sign レスポンスに signature フィールドがない"))?;
        // "vault:v1:" プレフィックスを除去して Base64 部分だけを取り出す
        let sig_b64 = sig_str
            .strip_prefix("vault:v1:")
            .unwrap_or(sig_str);
        // Base64 デコードして署名バイト列を返す
        let sig_bytes = BASE64_STD.decode(sig_b64)
            .context("OpenBao Transit signature の Base64 デコードに失敗した")?;
        Ok(sig_bytes)
    }

    /// verify は payload と signature を OpenBao Transit の key_name で検証して結果を返す。
    /// POST /v1/transit/verify/{key_name} を呼び出す。
    #[instrument(skip(self, payload, signature), fields(key_name = %key_name))]
    pub async fn verify(&self, key_name: &str, payload: &[u8], signature: &[u8]) -> Result<bool> {
        // payload を Base64 エンコードする（OpenBao Transit の input フィールドは Base64 要求）
        let input_b64 = BASE64_STD.encode(payload);
        // signature を "vault:v1:<base64>" 形式にエンコードする
        let sig_b64 = format!("vault:v1:{}", BASE64_STD.encode(signature));
        // verify エンドポイント URL を構築する
        let url = format!("{}/v1/transit/verify/{}", self.base_url, key_name);
        debug!(url = %url, "OpenBao Transit verify 呼び出し");
        // POST リクエストを送信する
        let resp = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&serde_json::json!({ "input": input_b64, "signature": sig_b64 }))
            .send()
            .await
            .context("OpenBao Transit verify リクエスト送信に失敗した")?;
        // HTTP ステータスを確認する
        if !resp.status().is_success() {
            // エラーステータス時はボディを取得してエラーメッセージに含める
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenBao Transit verify 失敗: HTTP {} body={}", status, body));
        }
        // レスポンス JSON をパースする
        let data: Value = resp.json().await
            .context("OpenBao Transit verify レスポンスの JSON パースに失敗した")?;
        // valid フィールドを取得して返す（存在しない場合は false とする）
        let valid = data["data"]["valid"].as_bool().unwrap_or(false);
        Ok(valid)
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // new_from_env が OPENBAO_TOKEN なしでエラーを返すことを確認する
    fn test_new_from_env_missing_token_returns_error() {
        // OPENBAO_TOKEN を未設定の状態で new_from_env を呼び出す
        // 環境変数は他のテストが設定している可能性があるため、削除してから確認する
        std::env::remove_var("OPENBAO_TOKEN");
        // OPENBAO_TOKEN が未設定の場合はエラーを返すことを確認する
        let result = OpenBaoTransitClient::new_from_env();
        assert!(result.is_err(), "OPENBAO_TOKEN 未設定時は Err を返すべき");
    }
}
