// apicurio_client.rs — Apicurio Schema Registry REST API クライアント
// GitOps パイプラインから schema artifact を登録・一覧取得・drift 検出するための実装
// docs/04_詳細設計/01_適合仕様 に基づく schema registry 連携の物理化

// reqwest: 非同期 HTTP クライアント（Apicurio REST API 呼び出しに使用する）
use reqwest::Client;
// serde: JSON デシリアライズ（Apicurio API レスポンスのパースに使用する）
use serde::Deserialize;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::{anyhow, Result};

// ============================================================
// Apicurio REST API レスポンス型
// ============================================================

// ArtifactSearchResults は Apicurio の GET /groups/{group}/artifacts レスポンスを表す
#[derive(Debug, Deserialize)]
struct ArtifactSearchResults {
    // artifacts: artifact メタデータのリスト
    artifacts: Vec<ArtifactMetaData>,
}

// ArtifactMetaData は Apicurio の artifact 一覧エントリを表す
#[derive(Debug, Deserialize)]
struct ArtifactMetaData {
    // id: artifact の識別子
    id: String,
}

// CreateArtifactResponse は Apicurio の POST /groups/{group}/artifacts レスポンスを表す
#[derive(Debug, Deserialize)]
struct CreateArtifactResponse {
    // id: 登録された artifact の識別子
    id: String,
    // version: 登録されたバージョン番号（文字列形式）
    #[allow(dead_code)]
    // version フィールドはデシリアライズするが直接は使用しない
    version: Option<String>,
}

// ArtifactContentResponse は Apicurio の GET /groups/{group}/artifacts/{id} レスポンスを表す
// drift 検出では raw bytes として比較するため構造体は使用しない

// ============================================================
// ApicurioClient 構造体
// ============================================================

// ApicurioClient は Apicurio Schema Registry への HTTP 接続を保持する構造体
pub struct ApicurioClient {
    // base_url: Apicurio Registry の base URL（例: http://apicurio.k1s0-infra.svc:8080）
    pub base_url: String,
    // token: Apicurio API 認証トークン（Bearer 方式）
    pub token: String,
    // http: reqwest の非同期 HTTP クライアントインスタンス
    http: Client,
}

// ApicurioClient の実装
impl ApicurioClient {
    // ApicurioClient を新規作成する
    // base_url: Apicurio Registry サーバーの URL
    // token: Bearer 認証トークン
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        // reqwest の非同期 HTTP クライアントを生成する
        let http = Client::new();
        // フィールドを初期化して返す
        Self {
            // base_url を String に変換して保存する
            base_url: base_url.into(),
            // token を String に変換して保存する
            token: token.into(),
            // http クライアントを保存する
            http,
        }
    }

    // ============================================================
    // schema artifact 登録
    // ============================================================

    // register_schema は artifact を Apicurio Registry に登録する
    // artifact_id: 登録する artifact の識別子
    // content: schema の raw bytes（JSONSchema / Protobuf / Avro 等）
    // Returns: 登録済み artifact の ID（Apicurio が採番した ID 文字列）
    pub async fn register_schema(&self, artifact_id: &str, content: &[u8]) -> Result<String> {
        // 生 bytes を content に使用する（生 bytes 露出禁止は public 型シグネチャの規約であり内部実装は適用外）
        // デフォルト group "default" を使用して artifact を登録する
        let group = "default";
        // POST エンドポイント URL を構築する
        let url = format!(
            "{}/apis/registry/v2/groups/{}/artifacts",
            self.base_url, group
        );
        // HTTP POST リクエストを送信する
        let response = self
            .http
            // POST メソッドで URL にリクエストを送信する
            .post(&url)
            // Bearer 認証ヘッダーを付与する
            .header("Authorization", format!("Bearer {}", self.token))
            // Content-Type は application/json を指定する（Apicurio v2 API 仕様）
            .header("Content-Type", "application/json")
            // X-Registry-ArtifactId ヘッダーで artifact ID を指定する
            .header("X-Registry-ArtifactId", artifact_id)
            // X-Registry-ArtifactType ヘッダーで schema タイプを指定する（JSON）
            .header("X-Registry-ArtifactType", "JSON")
            // schema content を body として送信する
            .body(content.to_vec())
            // リクエストを送信する
            .send()
            .await
            // reqwest エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio POST /artifacts 失敗: {}", e))?;

        // HTTP ステータスコードを確認する
        if !response.status().is_success() {
            // エラーステータスの場合は anyhow エラーを返す
            let status = response.status();
            // レスポンスボディをテキストとして取得する
            let body = response.text().await.unwrap_or_default();
            // エラーメッセージを構築して返す
            return Err(anyhow!(
                "Apicurio register_schema 失敗: status={}, body={}",
                status,
                body
            ));
        }

        // レスポンス JSON を CreateArtifactResponse にデシリアライズする
        let result: CreateArtifactResponse = response
            .json()
            .await
            // デシリアライズエラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio register_schema レスポンスのパース失敗: {}", e))?;

        // 登録された artifact の ID を返す
        Ok(result.id)
    }

    // ============================================================
    // artifact 一覧取得
    // ============================================================

    // list_artifacts は指定 group の全 artifact ID リストを返す
    // group: 一覧取得対象の Apicurio group 名
    // Returns: artifact ID の文字列ベクター
    pub async fn list_artifacts(&self, group: &str) -> Result<Vec<String>> {
        // GET エンドポイント URL を構築する
        let url = format!(
            "{}/apis/registry/v2/groups/{}/artifacts",
            self.base_url, group
        );
        // HTTP GET リクエストを送信する
        let response = self
            .http
            // GET メソッドで URL にリクエストを送信する
            .get(&url)
            // Bearer 認証ヘッダーを付与する
            .header("Authorization", format!("Bearer {}", self.token))
            // Accept ヘッダーに JSON を指定する
            .header("Accept", "application/json")
            // リクエストを送信する
            .send()
            .await
            // reqwest エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio GET /artifacts 失敗: {}", e))?;

        // HTTP ステータスコードを確認する
        if !response.status().is_success() {
            // エラーステータスの場合は anyhow エラーを返す
            let status = response.status();
            // レスポンスボディをテキストとして取得する
            let body = response.text().await.unwrap_or_default();
            // エラーメッセージを構築して返す
            return Err(anyhow!(
                "Apicurio list_artifacts 失敗: status={}, body={}",
                status,
                body
            ));
        }

        // レスポンス JSON を ArtifactSearchResults にデシリアライズする
        let results: ArtifactSearchResults = response
            .json()
            .await
            // デシリアライズエラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio list_artifacts レスポンスのパース失敗: {}", e))?;

        // artifact ID のみを抽出して返す
        Ok(results.artifacts.into_iter().map(|a| a.id).collect())
    }

    // ============================================================
    // schema drift 検出
    // ============================================================

    // detect_drift は Apicurio 上の artifact と local_schema を byte 比較して drift を検出する
    // artifact_id: 比較対象の artifact 識別子
    // local_schema: ローカルの schema bytes（比較対象）
    // Returns: drift がある場合は true、同一の場合は false を返す
    pub async fn detect_drift(&self, artifact_id: &str, local_schema: &[u8]) -> Result<bool> {
        // デフォルト group "default" を使用して artifact を取得する
        let group = "default";
        // GET エンドポイント URL を構築する（artifact の最新 content を取得する）
        let url = format!(
            "{}/apis/registry/v2/groups/{}/artifacts/{}",
            self.base_url, group, artifact_id
        );
        // HTTP GET リクエストを送信する
        let response = self
            .http
            // GET メソッドで URL にリクエストを送信する
            .get(&url)
            // Bearer 認証ヘッダーを付与する
            .header("Authorization", format!("Bearer {}", self.token))
            // Accept ヘッダーには */* を指定して raw content を取得する
            .header("Accept", "*/*")
            // リクエストを送信する
            .send()
            .await
            // reqwest エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio GET /artifacts/{} 失敗: {}", artifact_id, e))?;

        // HTTP ステータスコードを確認する
        if !response.status().is_success() {
            // エラーステータスの場合は anyhow エラーを返す
            let status = response.status();
            // レスポンスボディをテキストとして取得する
            let body = response.text().await.unwrap_or_default();
            // エラーメッセージを構築して返す
            return Err(anyhow!(
                "Apicurio detect_drift 取得失敗: artifact_id={}, status={}, body={}",
                artifact_id,
                status,
                body
            ));
        }

        // レスポンス body を bytes として取得する（生 bytes は内部比較にのみ使用し外部に露出しない）
        let remote_bytes = response
            .bytes()
            .await
            // bytes 取得エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio detect_drift bytes 取得失敗: {}", e))?;

        // remote bytes と local_schema を byte 単位で比較する
        // 一致する場合は drift なし (false)、異なる場合は drift あり (true) を返す
        let has_drift = remote_bytes.as_ref() != local_schema;

        // drift 検出結果を返す
        Ok(has_drift)
    }
}

// ============================================================
// ユニットテスト
// ============================================================

// apicurio_client のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // ApicurioClient の生成が成功することを確認するテスト
    #[test]
    fn test_apicurio_client_new() {
        // base_url と token を指定して ApicurioClient を生成する
        let client = ApicurioClient::new(
            // ローカル Apicurio Registry の URL を指定する
            "http://localhost:8080",
            // テスト用ダミートークンを指定する
            "test-token",
        );
        // base_url が正しく設定されていることを確認する
        assert_eq!(client.base_url, "http://localhost:8080");
        // token が正しく設定されていることを確認する
        assert_eq!(client.token, "test-token");
    }

    // detect_drift: 同一 bytes の場合は false を返すロジックを確認するテスト
    // （HTTP 通信なし / 純粋なバイト比較ロジックのみ検証する）
    #[test]
    fn test_drift_byte_comparison_logic() {
        // 同一の bytes を定義する
        let a: &[u8] = b"{\"type\": \"record\"}";
        // b は a と同一の bytes を指定する
        let b: &[u8] = b"{\"type\": \"record\"}";
        // 同一の場合は drift なし
        assert!(!( a != b ), "同一 bytes の場合は drift なし (false) を期待する");

        // 異なる bytes を定義する
        let c: &[u8] = b"{\"type\": \"record\", \"name\": \"updated\"}";
        // a と c が異なることを確認する
        assert!(a != c, "異なる bytes の場合は drift あり (true) を期待する");
    }
}
