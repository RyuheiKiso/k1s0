// key_handle.rs — spec 05 鍵管理適合仕様: KeyHandle opaque 型
// 04_認証適合仕様.md §v1 auth_class と 05_鍵管理適合仕様.md §v1 key_class に基づく。
// 公開 API シグネチャに生 key bytes を露出しない opaque 型を実装する。
// KeyMaterial は zeroize で drop 時にメモリをゼロクリアする。
// OpenBaoTransitClient: OpenBao Transit API の sign/verify/wrap/unwrap ラッパーを実装する。
// TenantTokenBucket: テナント容量適合仕様 09 の per-tenant token bucket を実装する。

// fmt: Display / Debug トレイト実装のための標準フォーマットモジュール
use std::fmt;
// serde: JSON シリアライズ / デシリアライズ（KeyHandle の HTTP 転送用）
use serde::{Deserialize, Serialize};
// zeroize: 機密データの drop 時ゼロクリア（KeyMaterial に適用）
use zeroize::Zeroize;
// base64: payload を OpenBao Transit API の input フィールド用に Base64 エンコードする
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
// std::env: 環境変数から OpenBao 接続設定を読み込む
use std::env;
// governor: per-tenant token bucket rate limiter（テナント容量適合仕様 09 の library_token_bucket）
use governor::{Quota, RateLimiter, DefaultKeyedRateLimiter};
// std::num::NonZeroU32: governor の Quota::per_second に必要な非ゼロ型
use std::num::NonZeroU32;
// std::sync::Arc: TenantTokenBucket の limiter を複数スレッドで共有する
use std::sync::Arc;

// KeyClass は 05_鍵管理適合仕様.md §v1 key_class セット（5 class）を宣言する。
// class 1 値が key_usage / rotation_policy / storage_backend を一意に導出する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyClass {
    // v1_data_dek: データ暗号化鍵（DEK）— テナントデータの AES-256-GCM 暗号化
    V1DataDek,
    // v1_data_kek: 鍵暗号化鍵（KEK）— DEK を wrap する HSM/OpenBao Transit 管理鍵
    V1DataKek,
    // v1_token_signing: トークン署名鍵 — JWT / DPoP proof 署名用 EC / EdDSA 秘密鍵
    V1TokenSigning,
    // v1_audit_root_signing: 監査ログ root 署名鍵 — audit chain の信頼アンカー
    V1AuditRootSigning,
    // v1_mtls_workload: workload mTLS 鍵 — SPIRE SVID 由来の TLS クライアント証明書秘密鍵
    V1MtlsWorkload,
}

impl fmt::Display for KeyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // KeyClass の文字列表現を返す（spec の class 名と 1:1 対応）
        match self {
            KeyClass::V1DataDek => write!(f, "v1_data_dek"),
            KeyClass::V1DataKek => write!(f, "v1_data_kek"),
            KeyClass::V1TokenSigning => write!(f, "v1_token_signing"),
            KeyClass::V1AuditRootSigning => write!(f, "v1_audit_root_signing"),
            KeyClass::V1MtlsWorkload => write!(f, "v1_mtls_workload"),
        }
    }
}

// KeyMaterial は実際の key bytes を保持する。drop 時に zeroize でゼロクリアする。
// KeyHandle の内部にのみ存在し、公開 API から見えない。
// Debug impl は key_bytes を "[REDACTED]" で隠蔽する。
#[derive(Zeroize)]
#[zeroize(drop)]
struct KeyMaterial {
    // key_bytes: 実際の暗号化鍵バイト列（AES-256 = 32 bytes, EC-P256 = 32 bytes etc.）
    key_bytes: Vec<u8>,
}

impl fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // key_bytes の内容を出力しない（[REDACTED] で隠蔽する）
        f.debug_struct("KeyMaterial")
            .field("key_bytes", &"[REDACTED]")
            .finish()
    }
}

// KeyHandle は生 key bytes を公開しない opaque 型。
// 05_鍵管理適合仕様.md の "KeyHandle 必須型" 規律を Rust 型システムで実装する。
// Serialize は handle_id / key_class のみを expose し、key_bytes は含まない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHandle {
    // handle_id: OpenBao Transit の key version identifier（UUID v7 形式）
    pub handle_id: String,
    // key_class: 鍵の用途クラス（5 class のいずれか）
    pub key_class: KeyClass,
    // is_valid: OpenBao による鍵の有効性確認結果（revoke / rotate 後 false になる）
    pub is_valid: bool,
    // raw bytes は Serialize に含まれない（serde(skip) で隠蔽する）
    #[serde(skip)]
    _material: Option<Arc<KeyMaterial>>,
}

impl KeyHandle {
    // create_stub は OpenBao Transit 呼出なしに stub の KeyHandle を生成する。
    // 本番では OpenBaoClient::encrypt/wrap を呼び出す bfl/src/openbao.rs を使う。
    pub fn create_stub(key_class: KeyClass, handle_id: String) -> Self {
        // 生 key bytes は受け取っても Self には保存せず、公開 API に漏れない
        Self {
            handle_id,
            key_class,
            is_valid: true,
            _material: None,
        }
    }

    // from_key_material は生 key bytes を受け取り、KeyHandle として wrap する。
    // 呼び出し元のスコープを抜けると key_bytes は zeroize で消去される。
    pub fn from_key_material(key_class: KeyClass, handle_id: String, key_bytes: Vec<u8>) -> Self {
        // KeyMaterial に key_bytes を移動（Arc 共有で複数サービスに渡せる）
        let material = Arc::new(KeyMaterial { key_bytes });
        Self {
            handle_id,
            key_class,
            is_valid: true,
            _material: Some(material),
        }
    }

    // key_class_str は key_class の文字列表現を返す（Serialize 済みフィールドのヘルパー）
    pub fn key_class_str(&self) -> String {
        // Display impl を使って文字列に変換する
        self.key_class.to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OpenBao Transit API レスポンス型（serde_json でデシリアライズする）
// ─────────────────────────────────────────────────────────────────────────────

// TransitSignResponse: POST /v1/{mount}/sign/{key} のレスポンス構造体
#[derive(Debug, Deserialize)]
struct TransitSignResponse {
    // data: OpenBao Transit API レスポンスのデータオブジェクト
    data: TransitSignData,
}

// TransitSignData: sign エンドポイントの data フィールド
#[derive(Debug, Deserialize)]
struct TransitSignData {
    // signature: vault:v1:{base64} 形式の署名文字列
    signature: String,
}

// TransitVerifyResponse: POST /v1/{mount}/verify/{key} のレスポンス構造体
#[derive(Debug, Deserialize)]
struct TransitVerifyResponse {
    // data: OpenBao Transit API レスポンスのデータオブジェクト
    data: TransitVerifyData,
}

// TransitVerifyData: verify エンドポイントの data フィールド
#[derive(Debug, Deserialize)]
struct TransitVerifyData {
    // valid: 署名検証結果（true = 有効, false = 無効）
    valid: bool,
}

// TransitEncryptResponse: POST /v1/{mount}/encrypt/{key} のレスポンス構造体
#[derive(Debug, Deserialize)]
struct TransitEncryptResponse {
    // data: OpenBao Transit API レスポンスのデータオブジェクト
    data: TransitEncryptData,
}

// TransitEncryptData: encrypt エンドポイントの data フィールド
#[derive(Debug, Deserialize)]
struct TransitEncryptData {
    // ciphertext: vault:v1:{base64} 形式の暗号文字列
    ciphertext: String,
}

// TransitDecryptResponse: POST /v1/{mount}/decrypt/{key} のレスポンス構造体
#[derive(Debug, Deserialize)]
struct TransitDecryptResponse {
    // data: OpenBao Transit API レスポンスのデータオブジェクト
    data: TransitDecryptData,
}

// TransitDecryptData: decrypt エンドポイントの data フィールド
#[derive(Debug, Deserialize)]
struct TransitDecryptData {
    // plaintext: Base64 エンコードされた復号済みペイロード
    plaintext: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// OpenBaoTransitClient: OpenBao Transit secrets engine の HTTP クライアント
// ─────────────────────────────────────────────────────────────────────────────

// OpenBaoTransitClient は OpenBao Transit API の sign/verify/wrap/unwrap を実装する。
// TLS なし（envoy サービスメッシュが mTLS を終端する）で動作する。
// 05_鍵管理適合仕様.md §transit_client の実装クラス。
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
        // parsed: JSON レスポンスを TransitSignResponse 構造体にデシリアライズする
        let parsed: TransitSignResponse = response.json().await?;
        // signature_str: vault:v1:{base64} 形式の署名文字列
        let signature_str = parsed.data.signature;
        // raw_b64: "vault:v1:" プレフィックスを除去して Base64 部分を取得する
        let raw_b64 = signature_str
            // "vault:v1:" プレフィックスを除去する（存在しない場合は元の文字列を使う）
            .strip_prefix("vault:v1:")
            // unwrap_or: プレフィックスがない場合は元の文字列をそのまま使う
            .unwrap_or(&signature_str);
        // decoded: Base64 デコードして署名バイト列を取得する
        let decoded = BASE64.decode(raw_b64)?;
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
        // parsed: JSON レスポンスを TransitVerifyResponse 構造体にデシリアライズする
        let parsed: TransitVerifyResponse = response.json().await?;
        // valid フィールドの bool 値を返す（true = 署名有効, false = 無効）
        Ok(parsed.data.valid)
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
        // parsed: JSON レスポンスを TransitEncryptResponse 構造体にデシリアライズする
        let parsed: TransitEncryptResponse = response.json().await?;
        // ciphertext_str: vault:v1:{base64} 形式の暗号文字列を取得する
        let ciphertext_str = parsed.data.ciphertext;
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
            anyhow::bail!("OpenBao unwrap (decrypt) failed: HTTP {} — {}", status, body_text);
        }
        // parsed: JSON レスポンスを TransitDecryptResponse 構造体にデシリアライズする
        let parsed: TransitDecryptResponse = response.json().await?;
        // plaintext_b64: Base64 エンコードされた復号済みペイロードを取得する
        let plaintext_b64 = parsed.data.plaintext;
        // decoded: Base64 デコードして平文バイト列を取得する
        let decoded = BASE64.decode(&plaintext_b64)?;
        // 平文バイト列を返す
        Ok(decoded)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TenantTokenBucket: テナント容量適合仕様 09 per-tenant token bucket
// ─────────────────────────────────────────────────────────────────────────────

// TenantTokenBucket はテナント容量適合仕様 09 の library_token_bucket を実装する。
// governor crate の DefaultKeyedRateLimiter<String> を使って per-tenant rate を管理する。
// check_and_consume は token 消費に成功した場合 true、rate limit 超過時 false を返す。
pub struct TenantTokenBucket {
    // limiter: テナント ID をキーとした per-tenant GCRA rate limiter
    // DefaultKeyedRateLimiter<String> は DashMap をバックエンドとしてスレッドセーフに動作する
    limiter: Arc<DefaultKeyedRateLimiter<String>>,
}

impl TenantTokenBucket {
    // new は tokens_per_second を上限とする per-tenant token bucket を生成する。
    // governor の Quota::per_second で 1 秒あたりのトークン上限を設定する。
    pub fn new(
        // tokens_per_second: テナントごとの 1 秒あたりのトークン許容数（0 は panic）
        tokens_per_second: u32,
    ) -> Self {
        // nz: NonZeroU32 に変換する（0 は panic; caller が正値を保証する）
        let nz = NonZeroU32::new(tokens_per_second)
            // 0 が渡された場合は即 panic してミスコンフィグを検出する
            .expect("tokens_per_second must be > 0");
        // quota: 1 秒あたり tokens_per_second トークンの GCRA Quota を定義する
        let quota = Quota::per_second(nz);
        // keyed: テナント ID をキーとした DefaultKeyedRateLimiter を生成する
        let keyed = RateLimiter::keyed(quota);
        // limiter を Arc で包んで clone 可能にする
        Self { limiter: Arc::new(keyed) }
    }

    // check_and_consume は tenant_id に対してトークンを 1 つ消費しようとする。
    // 消費に成功（rate limit 内）なら true を返す。
    // rate limit 超過なら false を返す（caller は HTTP 429 等を返す）。
    pub async fn check_and_consume(
        &self,
        // tenant_id: トークンを消費するテナントの識別子（UUID v4 文字列等）
        tenant_id: &str,
    ) -> bool {
        // check_key: tenant_id キーでトークンを 1 つ消費する
        // Ok(()) = 消費成功, Err(_) = rate limit 超過
        self.limiter
            // String キーで check_key を呼び出す
            .check_key(&tenant_id.to_string())
            // is_ok(): Ok なら true, Err なら false を返す
            .is_ok()
    }
}
