// envelope_encryption.rs — AES-256-GCM エンベロープ暗号化実装
// 28_業務添付帳票資産.md §暗号化要件 に準拠する
// DEK は OpenBao Transit Wrap API で KEK ラップし、平文 DEK は Zeroizing で自動消去する
// wall-clock TTL 禁止規約準拠: タイムスタンプ系フィールドには HLC を使用する
// 生 DEK / 生 access_token は公開 API に露出させない

// aes_gcm: AES-256-GCM 暗号化ライブラリ
use aes_gcm::{
    // Aes256Gcm: AES-256-GCM の暗号化アルゴリズム型
    Aes256Gcm,
    // Key: AES-256-GCM の鍵型（32 バイト）
    Key,
    // Nonce: AES-256-GCM の 12 バイト nonce 型
    Nonce,
    // KeyInit: 鍵から暗号化器を生成するトレイト（aes-gcm 0.10 で NewAead から改名）
    KeyInit,
    // Aead: encrypt / decrypt メソッドを提供するトレイト
    aead::Aead,
};
// anyhow: Result / Context / bail! によるエラーハンドリング
use anyhow::{Context, Result, bail};
// base64: nonce および DEK を JSON フィールドとして Base64 エンコード/デコードする
use base64::{engine::general_purpose::STANDARD as B64, Engine};
// rand: OsRng による暗号学的安全な乱数生成（nonce / DEK 生成に使用する）
use rand::RngCore;
// reqwest: OpenBao Transit API を呼び出す非同期 HTTP クライアント
use reqwest::Client as HttpClient;
// serde: JSON リクエスト/レスポンスのシリアライズに使用する
use serde::{Deserialize, Serialize};
// zeroize: DEK をスコープ終了時に確実にゼロ消去するためのラッパー型
use zeroize::Zeroizing;

// CiphertextEnvelope は暗号化結果と付随するメタデータを保持する構造体
// この型を公開 API で返す際、wrapped_dek は base64 エンコード済み文字列であり生 DEK ではない
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiphertextEnvelope {
    // AES-256-GCM 暗号文（nonce 12B + ciphertext + GCM tag 16B が結合された形式）
    pub ciphertext: Vec<u8>,
    // OpenBao Transit が KEK でラップした DEK（"vault:v1:..." 形式の base64 文字列）
    // 生 DEK バイト列を公開 API に露出することを禁止する
    pub wrapped_dek: String,
    // AES-256-GCM の 12 バイト nonce（Base64 エンコード済み）
    pub nonce_b64: String,
    // OpenBao Transit の鍵名（どの KEK を使ったか追跡するために保持する）
    pub key_name: String,
    // OpenBao Transit の鍵バージョン番号（KEK ローテーション後も旧バージョンで復号できる）
    pub key_version: u32,
}

// OpenBao Transit Encrypt API へのリクエスト構造体
// POST /v1/transit/encrypt/{key_name} に送信する JSON ペイロード
#[derive(Serialize)]
struct TransitEncryptRequest {
    // plaintext: Base64 エンコードされた平文（OpenBao Transit は base64 入力を要求する）
    plaintext: String,
}

// OpenBao Transit Encrypt API のレスポンス構造体
// {"data": {"ciphertext": "vault:v1:...", "key_version": 1}} の data フィールド
#[derive(Deserialize)]
struct TransitEncryptData {
    // ciphertext: OpenBao が生成した暗号文（"vault:v1:..." 形式）
    ciphertext: String,
    // key_version: 使用された KEK のバージョン番号
    key_version: u32,
}

// OpenBao Transit Encrypt API のトップレベルレスポンス
#[derive(Deserialize)]
struct TransitEncryptResponse {
    // data: 暗号化結果を含むフィールド
    data: TransitEncryptData,
}

// OpenBao Transit Decrypt API へのリクエスト構造体
// POST /v1/transit/decrypt/{key_name} に送信する JSON ペイロード
#[derive(Serialize)]
struct TransitDecryptRequest {
    // ciphertext: OpenBao Transit の暗号文形式（"vault:v1:..." 文字列）
    ciphertext: String,
}

// OpenBao Transit Decrypt API のレスポンスの data フィールド
#[derive(Deserialize)]
struct TransitDecryptData {
    // plaintext: Base64 エンコードされた復号済み平文（DEK バイト列を base64 にしたもの）
    plaintext: String,
}

// OpenBao Transit Decrypt API のトップレベルレスポンス
#[derive(Deserialize)]
struct TransitDecryptResponse {
    // data: 復号結果を含むフィールド
    data: TransitDecryptData,
}

// EnvelopeEncryptor は AES-256-GCM + OpenBao Transit によるエンベロープ暗号化/復号を実装する
// 生 DEK はこの型の外部には一切公開しない（Zeroizing<Vec<u8>> で保護する）
pub struct EnvelopeEncryptor {
    // OpenBao の API ベース URL（例: "http://openbao:8200"）
    // 環境変数 OPENBAO_ADDR から取得する
    openbao_addr: String,
    // OpenBao API アクセストークン（環境変数 OPENBAO_TOKEN から取得する）
    // 生 token を公開フィールドにしないために private にする
    openbao_token: String,
    // reqwest 非同期 HTTP クライアント（再利用によりコネクションプールを活用する）
    http: HttpClient,
}

impl EnvelopeEncryptor {
    // new はコンストラクタ: 環境変数 OPENBAO_ADDR / OPENBAO_TOKEN から設定を読み込む
    // 環境変数が未設定の場合はエラーを返す
    pub fn new() -> Result<Self> {
        // OPENBAO_ADDR 環境変数を読み込む（未設定は設定不備エラー）
        let openbao_addr = std::env::var("OPENBAO_ADDR")
            .context("OPENBAO_ADDR 環境変数が設定されていません")?;
        // OPENBAO_TOKEN 環境変数を読み込む（未設定は設定不備エラー）
        let openbao_token = std::env::var("OPENBAO_TOKEN")
            .context("OPENBAO_TOKEN 環境変数が設定されていません")?;
        // reqwest HTTP クライアントを生成する
        let http = HttpClient::new();
        // 設定を保持するインスタンスを返す
        Ok(Self {
            openbao_addr,
            openbao_token,
            http,
        })
    }

    // encrypt は平文バイト列を AES-256-GCM で暗号化し、DEK を OpenBao Transit KEK でラップする
    // plaintext: 暗号化対象バイト列（PII フィールドを含む可能性がある）
    // key_name: OpenBao Transit の鍵名（テナントごとまたはグローバル共有鍵を指定する）
    // 返値: CiphertextEnvelope（生 DEK は含まない）
    pub async fn encrypt(
        &self,
        plaintext: &[u8],
        key_name: &str,
    ) -> Result<CiphertextEnvelope> {
        // 32 バイトの DEK を OsRng で生成する（AES-256 には 32 バイトが必要）
        // Zeroizing でラップすることでスコープ終了時に確実にゼロ消去する
        let mut dek_bytes = Zeroizing::new(vec![0u8; 32]);
        // OsRng でランダムな DEK バイト列を生成する
        rand::rngs::OsRng.fill_bytes(dek_bytes.as_mut_slice());

        // 12 バイト（96 ビット）の nonce を OsRng で生成する（AES-256-GCM 推奨値）
        let mut nonce_bytes = [0u8; 12];
        // OsRng で暗号学的安全な nonce を生成する
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

        // AES-256-GCM の鍵を DEK バイト列から生成する（aes-gcm 0.10 では Key::<Aes256Gcm>::from_slice を使用する）
        let key = Key::<Aes256Gcm>::from_slice(dek_bytes.as_slice());
        // AES-256-GCM 暗号化器を生成する（KeyInit トレイトの new を使用する）
        let cipher = Aes256Gcm::new(key);
        // nonce 型を生成する
        let nonce = Nonce::from_slice(&nonce_bytes);
        // AES-256-GCM で平文を暗号化する（失敗時はエラーを返す）
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("AES-256-GCM 暗号化エラー: {}", e))?;

        // DEK を Base64 エンコードして OpenBao Transit Encrypt API に送信する
        // OpenBao Transit の plaintext フィールドは Base64 入力を要求する
        let dek_b64 = B64.encode(dek_bytes.as_slice());
        // OpenBao Transit Encrypt リクエストを構築する
        let transit_req = TransitEncryptRequest {
            plaintext: dek_b64,
        };
        // OpenBao Transit Encrypt API の URL を構築する
        let url = format!(
            "{}/v1/transit/encrypt/{}",
            self.openbao_addr, key_name
        );
        // OpenBao Transit Encrypt API を POST で呼び出す
        let resp = self
            .http
            .post(&url)
            // X-Vault-Token ヘッダーに OpenBao トークンを付与する
            .header("X-Vault-Token", &self.openbao_token)
            // JSON ボディでリクエストを送信する
            .json(&transit_req)
            .send()
            .await
            .context("OpenBao Transit Encrypt API への HTTP リクエストに失敗しました")?;

        // HTTP ステータスコードを確認し、エラー時は詳細メッセージを付けて返す
        let status = resp.status();
        // 2xx 以外のステータスは失敗とみなす
        if !status.is_success() {
            // エラーレスポンスのボディを取得する
            let body = resp.text().await.unwrap_or_default();
            // エラーメッセージを構築して返す
            bail!(
                "OpenBao Transit Encrypt API が失敗しました: status={}, body={}",
                status,
                body
            );
        }

        // レスポンスを TransitEncryptResponse として parse する
        let transit_resp: TransitEncryptResponse = resp
            .json()
            .await
            .context("OpenBao Transit Encrypt API レスポンスの JSON parse に失敗しました")?;

        // nonce を Base64 エンコードして envelope に含める（スライス参照は不要）
        let nonce_b64 = B64.encode(nonce_bytes);

        // CiphertextEnvelope を組み立てて返す（生 DEK は含まない）
        // Zeroizing<Vec<u8>> の dek_bytes はここで drop され自動ゼロ消去される
        Ok(CiphertextEnvelope {
            // AES-256-GCM 暗号文（nonce + ciphertext + GCM tag が結合された形式）
            ciphertext,
            // OpenBao Transit が生成した KEK ラップ済み DEK 文字列
            wrapped_dek: transit_resp.data.ciphertext,
            // Base64 エンコードされた 12 バイト nonce
            nonce_b64,
            // 使用した OpenBao Transit 鍵名
            key_name: key_name.to_string(),
            // 使用した KEK のバージョン番号
            key_version: transit_resp.data.key_version,
        })
    }

    // decrypt は CiphertextEnvelope を受け取り、OpenBao Transit で DEK を展開して平文を復号する
    // envelope: 復号対象の CiphertextEnvelope（encrypt が返した構造体と同じ形式）
    // 返値: 復号された平文バイト列
    pub async fn decrypt(&self, envelope: &CiphertextEnvelope) -> Result<Vec<u8>> {
        // OpenBao Transit Decrypt API の URL を構築する
        let url = format!(
            "{}/v1/transit/decrypt/{}",
            self.openbao_addr, envelope.key_name
        );
        // OpenBao Transit Decrypt リクエストを構築する（wrapped_dek を送信する）
        let transit_req = TransitDecryptRequest {
            ciphertext: envelope.wrapped_dek.clone(),
        };
        // OpenBao Transit Decrypt API を POST で呼び出す
        let resp = self
            .http
            .post(&url)
            // X-Vault-Token ヘッダーに OpenBao トークンを付与する
            .header("X-Vault-Token", &self.openbao_token)
            // JSON ボディでリクエストを送信する
            .json(&transit_req)
            .send()
            .await
            .context("OpenBao Transit Decrypt API への HTTP リクエストに失敗しました")?;

        // HTTP ステータスコードを確認し、エラー時は詳細メッセージを付けて返す
        let status = resp.status();
        // 2xx 以外のステータスは失敗とみなす
        if !status.is_success() {
            // エラーレスポンスのボディを取得する
            let body = resp.text().await.unwrap_or_default();
            // エラーメッセージを構築して返す
            bail!(
                "OpenBao Transit Decrypt API が失敗しました: status={}, body={}",
                status,
                body
            );
        }

        // レスポンスを TransitDecryptResponse として parse する
        let transit_resp: TransitDecryptResponse = resp
            .json()
            .await
            .context("OpenBao Transit Decrypt API レスポンスの JSON parse に失敗しました")?;

        // OpenBao Transit が返した Base64 エンコード済み DEK をデコードする
        // Zeroizing でラップしてスコープ終了時に確実にゼロ消去する
        let dek_bytes = Zeroizing::new(
            B64.decode(&transit_resp.data.plaintext)
                .context("OpenBao Transit Decrypt レスポンスの DEK base64 デコードに失敗しました")?,
        );

        // DEK の長さを検証する（AES-256 には 32 バイトが必要）
        if dek_bytes.len() != 32 {
            bail!(
                "DEK の長さが不正です: expected=32, actual={}",
                dek_bytes.len()
            );
        }

        // nonce を Base64 デコードする
        let nonce_bytes = B64.decode(&envelope.nonce_b64)
            .context("envelope の nonce base64 デコードに失敗しました")?;
        // nonce の長さを検証する（AES-256-GCM は 12 バイト）
        if nonce_bytes.len() != 12 {
            bail!(
                "nonce の長さが不正です: expected=12, actual={}",
                nonce_bytes.len()
            );
        }

        // AES-256-GCM の鍵を DEK バイト列から生成する（aes-gcm 0.10 では Key::<Aes256Gcm>::from_slice を使用する）
        let key = Key::<Aes256Gcm>::from_slice(dek_bytes.as_slice());
        // AES-256-GCM 復号器を生成する（KeyInit トレイトの new を使用する）
        let cipher = Aes256Gcm::new(key);
        // nonce 型を生成する
        let nonce = Nonce::from_slice(&nonce_bytes);
        // AES-256-GCM で暗号文を復号する（GCM タグで完全性も検証される）
        let plaintext = cipher
            .decrypt(nonce, envelope.ciphertext.as_slice())
            .map_err(|e| anyhow::anyhow!("AES-256-GCM 復号エラー（完全性検証失敗の可能性あり）: {}", e))?;

        // 復号された平文バイト列を返す
        // Zeroizing<Vec<u8>> の dek_bytes はここで drop され自動ゼロ消去される
        Ok(plaintext)
    }
}

// EncryptedEnvelope: 後方互換のために旧型定義を維持する（外部クレートが参照する場合に備える）
// 新しいコードでは CiphertextEnvelope を使用すること
#[deprecated(
    since = "0.1.0",
    note = "CiphertextEnvelope を使用してください。EnvelopeEncryptor::encrypt を参照。"
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    // KEK で暗号化された DEK（OpenBao Transit の暗号文形式）
    pub encrypted_dek: Vec<u8>,
    // DEK で暗号化された平文データ（AES-256-GCM）
    pub ciphertext: Vec<u8>,
    // AES-256-GCM の認証タグ（完全性検証に使用する）
    pub auth_tag: Vec<u8>,
    // AES-256-GCM の nonce（初期化ベクトル）
    pub nonce: Vec<u8>,
    // 使用した KEK のバージョン識別子（ローテーション後も旧 KEK で復号可能にする）
    pub kek_version: String,
}

// EnvelopeEncryption: 後方互換のために旧型定義を維持する
// 新しいコードでは EnvelopeEncryptor を使用すること
#[deprecated(
    since = "0.1.0",
    note = "EnvelopeEncryptor を使用してください。EnvelopeEncryptor::new を参照。"
)]
pub struct EnvelopeEncryption {
    // OpenBao Transit エンドポイント URL
    transit_endpoint: String,
    // 使用する Transit キー名
    key_name: String,
}

#[allow(deprecated)]
impl EnvelopeEncryption {
    // EnvelopeEncryption を生成する（旧 API）
    pub fn new(transit_endpoint: String, key_name: String) -> Self {
        // OpenBao Transit 接続情報を保持するインスタンスを生成する
        Self {
            transit_endpoint,
            key_name,
        }
    }

    // 旧 encrypt API: スタブとして残す（新規実装は EnvelopeEncryptor を使うこと）
    #[allow(clippy::unused_self)]
    pub fn encrypt(&self, _dek: &[u8], _plaintext: &[u8]) -> Result<EncryptedEnvelope> {
        // 旧 API は EnvelopeEncryptor に移行済みのため直接呼び出しは禁止する
        bail!(
            "EnvelopeEncryption は deprecated です。EnvelopeEncryptor::encrypt を使用してください。\
             transit_endpoint={}, key_name={}",
            self.transit_endpoint,
            self.key_name
        )
    }

    // 旧 decrypt API: スタブとして残す（新規実装は EnvelopeEncryptor を使うこと）
    #[allow(clippy::unused_self)]
    pub fn decrypt(&self, _envelope: &EncryptedEnvelope) -> Result<Vec<u8>> {
        // 旧 API は EnvelopeEncryptor に移行済みのため直接呼び出しは禁止する
        bail!(
            "EnvelopeEncryption は deprecated です。EnvelopeEncryptor::decrypt を使用してください。\
             transit_endpoint={}, key_name={}",
            self.transit_endpoint,
            self.key_name
        )
    }
}
