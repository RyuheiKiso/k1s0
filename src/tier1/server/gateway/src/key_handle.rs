// key_handle.rs — k1s0 tier1 gateway: KeyClass / KeyHandle の canonical 再エクスポート
// 独自定義を廃止し、k1s0-tier1-library の canonical 実装を参照する。
// CLAUDE.md「重複実装は drift リスクで禁止」規律の物理化。
// OpenBaoTransitClient は gateway 固有の HTTP クライアントとして本モジュールで提供する。

// k1s0-tier1-library の KeyClass を canonical 実装から再エクスポートする
pub use k1s0_tier1_library::key_handle::KeyClass;
// k1s0-tier1-library の OpenBaoKeyHandle を KeyHandle として再エクスポートする
// gateway の公開 API シグネチャ（handle: KeyHandle）との後方互換を保つ type alias
pub use k1s0_tier1_library::key_handle::OpenBaoKeyHandle as KeyHandle;

// base64: OpenBao Transit API の input フィールド用 Base64 エンコードに使用する
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
// std::env: 環境変数から OpenBao 接続設定を読み込む
use std::env;

// OpenBaoTransitClient は gateway が OpenBao Transit API を呼び出す HTTP クライアント。
// sign / verify / wrap / unwrap の 4 操作を提供する（05_鍵管理適合仕様.md §transit_client）。
// TLS なし（envoy サービスメッシュが mTLS を終端する）で動作する。
pub struct OpenBaoTransitClient {
    // base_url: OpenBao API base URL（例: http://openbao.svc:8200）
    base_url: String,
    // token: OpenBao API token（AppRole auth で取得した Vault トークン）
    token: String,
    // mount_path: Transit secrets engine のマウントパス（例: transit）
    mount_path: String,
    // http: TLS なしの reqwest HTTP クライアント（envoy が TLS 終端）
    http: reqwest::Client,
}

impl OpenBaoTransitClient {
    // from_env は環境変数から設定を読み込んで OpenBaoTransitClient を構築する。
    // OPENBAO_ADDR: OpenBao サーバーのアドレス（例: http://openbao.svc:8200）
    // OPENBAO_TOKEN: OpenBao API トークン（AppRole token）
    // OPENBAO_TRANSIT_MOUNT: Transit engine のマウントパス（省略時: transit）
    pub fn from_env() -> Self {
        // OPENBAO_ADDR: OpenBao サーバーアドレスを環境変数から取得する（未設定時は localhost:8200）
        let base_url = env::var("OPENBAO_ADDR")
            .unwrap_or_else(|_| "http://127.0.0.1:8200".to_string());
        // OPENBAO_TOKEN: OpenBao API トークンを環境変数から取得する（未設定時は空文字）
        let token = env::var("OPENBAO_TOKEN")
            .unwrap_or_default();
        // OPENBAO_TRANSIT_MOUNT: Transit engine マウントパスを環境変数から取得する（省略時: transit）
        let mount_path = env::var("OPENBAO_TRANSIT_MOUNT")
            .unwrap_or_else(|_| "transit".to_string());
        // reqwest::Client: TLS なし（envoy が mTLS 終端）でビルドする
        let http = reqwest::Client::builder()
            // timeout: OpenBao API 呼び出しのタイムアウトを 5 秒に設定する
            .timeout(std::time::Duration::from_secs(5))
            // build: クライアントを構築する（設定エラーは panic — 起動時に検出する）
            .build()
            .expect("reqwest::Client build failed");
        // 構築した OpenBaoTransitClient を返す
        Self { base_url, token, mount_path, http }
    }

    // sign は KeyHandle に対応する Transit signing key でペイロードに署名する。
    // OpenBao Transit API: POST /v1/{mount}/sign/{key_name}
    // リクエスト body: {"input": "<base64(payload)>"}
    // レスポンス: {"data": {"signature": "vault:v1:<base64_sig>"}}
    pub async fn sign(
        &self,
        // key_handle: 署名に使用する鍵の handle（handle_id が Transit key 名に対応する）
        key_handle: &KeyHandle,
        // payload: 署名対象のバイト列（生バイト — Base64 エンコードは本メソッドが行う）
        payload: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        // URL: POST /v1/{mount}/sign/{key_name} のフルパスを組み立てる
        let url = format!(
            "{}/v1/{}/sign/{}",
            self.base_url, self.mount_path, key_handle.handle_id
        );
        // input_b64: OpenBao Transit API が要求する Base64 エンコード済みペイロード
        let input_b64 = BASE64.encode(payload);
        // body: リクエスト JSON オブジェクト {"input": "<base64>"}
        let body = serde_json::json!({ "input": input_b64 });
        // response: OpenBao Transit API に POST して HTTP レスポンスを受け取る
        let response = self.http
            // POST リクエストを送信する
            .post(&url)
            // X-Vault-Token ヘッダーに OpenBao API トークンをセットする
            .header("X-Vault-Token", &self.token)
            // JSON body をセットする
            .json(&body)
            // 非同期で送信して HTTP レスポンスを受け取る
            .send()
            .await?;
        // status: HTTP ステータスコードを確認し、エラーなら anyhow::Error を返す
        let status = response.status();
        // status が 2xx でない場合はエラーとして返す
        if !status.is_success() {
            // レスポンスボディをテキストで取得してエラーメッセージに含める
            let body_text = response.text().await.unwrap_or_default();
            // anyhow::bail!: エラーを返す
            anyhow::bail!("OpenBao sign failed: HTTP {} — {}", status, body_text);
        }
        // parsed: JSON レスポンスを serde_json::Value にデシリアライズする
        let parsed: serde_json::Value = response.json().await?;
        // signature_str: vault:v1:{base64} 形式の署名文字列を取得する
        let signature_str = parsed["data"]["signature"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("OpenBao sign: signature フィールドが存在しない"))?
            .to_string();
        // raw_b64: "vault:v1:" プレフィックスを除去して Base64 部分を取得する
        let raw_b64 = signature_str
            // "vault:v1:" プレフィックスを除去する（存在しない場合は元の文字列を使う）
            .strip_prefix("vault:v1:")
            // unwrap_or: プレフィックスがない場合は元の文字列をそのまま使う
            .unwrap_or(&signature_str)
            .to_string();
        // decoded: Base64 デコードして署名バイト列を取得する
        let decoded = BASE64.decode(&raw_b64)?;
        // 署名バイト列を返す
        Ok(decoded)
    }

    // verify は Transit signing key で payload と signature の整合性を検証する。
    // OpenBao Transit API: POST /v1/{mount}/verify/{key_name}
    // リクエスト body: {"input": "<base64(payload)>", "signature": "vault:v1:<base64_sig>"}
    // レスポンス: {"data": {"valid": true/false}}
    pub async fn verify(
        &self,
        // key_handle: 検証に使用する鍵の handle（handle_id が Transit key 名に対応する）
        key_handle: &KeyHandle,
        // payload: 署名対象のバイト列（sign 時と同じバイト列を渡す）
        payload: &[u8],
        // signature: sign メソッドが返した生署名バイト列（Base64 エンコードはここで行う）
        signature: &[u8],
    ) -> anyhow::Result<bool> {
        // URL: POST /v1/{mount}/verify/{key_name} のフルパスを組み立てる
        let url = format!(
            "{}/v1/{}/verify/{}",
            self.base_url, self.mount_path, key_handle.handle_id
        );
        // input_b64: payload を Base64 エンコードする（OpenBao API 要件）
        let input_b64 = BASE64.encode(payload);
        // sig_b64: 生署名バイト列を Base64 エンコードして vault:v1: プレフィックスを付ける
        let sig_b64 = format!("vault:v1:{}", BASE64.encode(signature));
        // body: リクエスト JSON オブジェクト {"input": ..., "signature": ...}
        let body = serde_json::json!({
            "input": input_b64,
            "signature": sig_b64,
        });
        // response: OpenBao Transit API に POST して HTTP レスポンスを受け取る
        let response = self.http
            // POST リクエストを送信する
            .post(&url)
            // X-Vault-Token ヘッダーに OpenBao API トークンをセットする
            .header("X-Vault-Token", &self.token)
            // JSON body をセットする
            .json(&body)
            // 非同期で送信して HTTP レスポンスを受け取る
            .send()
            .await?;
        // status: HTTP ステータスコードを確認し、エラーなら anyhow::Error を返す
        let status = response.status();
        // status が 2xx でない場合はエラーとして返す
        if !status.is_success() {
            // レスポンスボディをテキストで取得してエラーメッセージに含める
            let body_text = response.text().await.unwrap_or_default();
            // anyhow::bail!: エラーを返す
            anyhow::bail!("OpenBao verify failed: HTTP {} — {}", status, body_text);
        }
        // parsed: JSON レスポンスを serde_json::Value にデシリアライズする
        let parsed: serde_json::Value = response.json().await?;
        // valid フィールドの bool 値を返す（true = 署名有効, false = 無効）
        Ok(parsed["data"]["valid"].as_bool().unwrap_or(false))
    }

    // wrap は Transit encryption で plaintext を暗号化する（DEK の KEK wrap に使用する）。
    // OpenBao Transit API: POST /v1/{mount}/encrypt/{key_name}
    // リクエスト body: {"plaintext": "<base64(plaintext)>"}
    // レスポンス: {"data": {"ciphertext": "vault:v1:<base64_ct>"}}
    pub async fn wrap(
        &self,
        // key_handle: 暗号化に使用する鍵の handle（V1DataKek クラスを想定する）
        key_handle: &KeyHandle,
        // plaintext: 暗号化対象の平文バイト列
        plaintext: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        // URL: POST /v1/{mount}/encrypt/{key_name} のフルパスを組み立てる
        let url = format!(
            "{}/v1/{}/encrypt/{}",
            self.base_url, self.mount_path, key_handle.handle_id
        );
        // plaintext_b64: OpenBao Transit API が要求する Base64 エンコード済み平文
        let plaintext_b64 = BASE64.encode(plaintext);
        // body: リクエスト JSON オブジェクト {"plaintext": "<base64>"}
        let body = serde_json::json!({ "plaintext": plaintext_b64 });
        // response: OpenBao Transit API に POST して HTTP レスポンスを受け取る
        let response = self.http
            // POST リクエストを送信する
            .post(&url)
            // X-Vault-Token ヘッダーに OpenBao API トークンをセットする
            .header("X-Vault-Token", &self.token)
            // JSON body をセットする
            .json(&body)
            // 非同期で送信して HTTP レスポンスを受け取る
            .send()
            .await?;
        // status: HTTP ステータスコードを確認し、エラーなら anyhow::Error を返す
        let status = response.status();
        // status が 2xx でない場合はエラーとして返す
        if !status.is_success() {
            // レスポンスボディをテキストで取得してエラーメッセージに含める
            let body_text = response.text().await.unwrap_or_default();
            // anyhow::bail!: エラーを返す
            anyhow::bail!("OpenBao wrap (encrypt) failed: HTTP {} — {}", status, body_text);
        }
        // parsed: JSON レスポンスを serde_json::Value にデシリアライズする
        let parsed: serde_json::Value = response.json().await?;
        // ciphertext_str: vault:v1:{base64} 形式の暗号文字列を取得する
        let ciphertext_str = parsed["data"]["ciphertext"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("OpenBao wrap: ciphertext フィールドが存在しない"))?
            .to_string();
        // 暗号文字列をバイト列に変換して返す（呼び出し元は vault:v1: プレフィックス込みで保存する）
        Ok(ciphertext_str.into_bytes())
    }

    // unwrap_key は Transit decryption で ciphertext を復号する（wrap の逆操作）。
    // OpenBao Transit API: POST /v1/{mount}/decrypt/{key_name}
    // リクエスト body: {"ciphertext": "vault:v1:<base64_ct>"}
    // レスポンス: {"data": {"plaintext": "<base64_plaintext>"}}
    pub async fn unwrap_key(
        &self,
        // key_handle: 復号に使用する鍵の handle（V1DataKek クラスを想定する）
        key_handle: &KeyHandle,
        // ciphertext: wrap メソッドが返したバイト列（vault:v1: プレフィックス込み）
        ciphertext: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        // URL: POST /v1/{mount}/decrypt/{key_name} のフルパスを組み立てる
        let url = format!(
            "{}/v1/{}/decrypt/{}",
            self.base_url, self.mount_path, key_handle.handle_id
        );
        // ciphertext_str: バイト列を UTF-8 文字列に変換する（vault:v1: プレフィックス込み）
        let ciphertext_str = String::from_utf8(ciphertext.to_vec())
            // from_utf8 エラーは anyhow::Error に変換する
            .map_err(|e| anyhow::anyhow!("ciphertext is not valid UTF-8: {}", e))?;
        // body: リクエスト JSON オブジェクト {"ciphertext": "vault:v1:..."}
        let body = serde_json::json!({ "ciphertext": ciphertext_str });
        // response: OpenBao Transit API に POST して HTTP レスポンスを受け取る
        let response = self.http
            // POST リクエストを送信する
            .post(&url)
            // X-Vault-Token ヘッダーに OpenBao API トークンをせっとする
            .header("X-Vault-Token", &self.token)
            // JSON body をセットする
            .json(&body)
            // 非同期で送信して HTTP レスポンスを受け取る
            .send()
            .await?;
        // status: HTTP ステータスコードを確認し、エラーなら anyhow::Error を返す
        let status = response.status();
        // status が 2xx でない場合はエラーとして返す
        if !status.is_success() {
            // レスポンスボディをテキストで取得してエラーメッセージに含める
            let body_text = response.text().await.unwrap_or_default();
            // anyhow::bail!: エラーを返す
            anyhow::bail!("OpenBao unwrap (decrypt) failed: HTTP {} — {}", status, body_text);
        }
        // parsed: JSON レスポンスを serde_json::Value にデシリアライズする
        let parsed: serde_json::Value = response.json().await?;
        // plaintext_b64: Base64 エンコードされた復号済みペイロードを取得する
        let plaintext_b64 = parsed["data"]["plaintext"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("OpenBao unwrap: plaintext フィールドが存在しない"))?
            .to_string();
        // decoded: Base64 デコードして平文バイト列を取得する
        let decoded = BASE64.decode(&plaintext_b64)?;
        // 平文バイト列を返す
        Ok(decoded)
    }
}
