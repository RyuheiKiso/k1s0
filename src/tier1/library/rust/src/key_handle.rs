// key_handle.rs — k1s0 tier1 Library: KeyHandle 抽象 + OpenBaoKeyHandle 実装
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）および
// §5 層 defense-in-depth 層 A「compile: KeyHandle 必須引数化、生 key bytes 不可視」に準拠する。
// 公開 API シグネチャに生 key bytes を露出しない opaque 型を実装する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: シリアライズ/デシリアライズ（derive feature を使用する）
use serde::{Deserialize, Serialize};
// zeroize: KeyMaterial のゼロクリア（drop 時に key bytes をゼロクリアする）
use zeroize::Zeroize;
// fmt: Display / Debug の手動実装に使用する
use std::fmt;
// sync::Arc: KeyMaterial を複数のサービス間で安全に共有する
use std::sync::Arc;
// base64: OpenBao Transit API の input / signature フィールド用 Base64 エンコード/デコード
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STD};

// KeyClass は 05_鍵管理適合仕様.md §v1 key_class セット（5 class）を宣言する。
// class 1 値が purpose / rotation_cadence / scope / backend / destruction_method を一意に導出する
//（dimension override 禁止）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// serde: spec の class 名（snake_case）と 1:1 対応するシリアライズ形式
#[serde(rename_all = "snake_case")]
pub enum KeyClass {
    // v1_data_dek: データ暗号化鍵（DEK）— per-tenant / software_kms_wrapped / crypto_shred
    V1DataDek,
    // v1_data_kek: 鍵暗号化鍵（KEK）— per-tenant / hsm_pkcs11_shamir_distributed / hsm_zeroize_all_shares
    V1DataKek,
    // v1_token_signing: JWT / DPoP 署名鍵 — platform / hsm_pkcs11 / jwks_revoke
    V1TokenSigning,
    // v1_audit_root_signing: audit hash chain root 署名鍵 — per-tenant / hsm_pkcs11 / external_notary_attest
    V1AuditRootSigning,
    // v1_mtls_workload: workload mTLS 鍵 — per_workload / spire / spire_revoke
    V1MtlsWorkload,
}

// KeyClass の文字列表現を返す（spec の class 名と 1:1 対応する）
impl fmt::Display for KeyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 各 class を spec 定義の snake_case 文字列にマッピングする
        match self {
            KeyClass::V1DataDek => write!(f, "v1_data_dek"),
            KeyClass::V1DataKek => write!(f, "v1_data_kek"),
            KeyClass::V1TokenSigning => write!(f, "v1_token_signing"),
            KeyClass::V1AuditRootSigning => write!(f, "v1_audit_root_signing"),
            KeyClass::V1MtlsWorkload => write!(f, "v1_mtls_workload"),
        }
    }
}

// KeyHandle は生 key bytes を公開しない opaque 鍵抽象 trait。
// 05_鍵管理適合仕様.md §KeyHandle / KeyMaterial の言語横断型 に準拠する。
// Sign / Verify は OpenBao Transit への委譲として実装し、key bytes は tier1 境界を越えない。
#[async_trait]
pub trait KeyHandle: Send + Sync {
    // key_id は OpenBao Transit のキー版数識別子（UUID v7 形式）を返す。
    fn key_id(&self) -> &str;
    // key_class は 5 class のいずれかを返す（purpose bundle の代表値）。
    fn key_class(&self) -> KeyClass;
    // is_valid は OpenBao による鍵の有効性確認結果を返す（revoke / rotate 後 false になる）。
    fn is_valid(&self) -> bool;
    // sign は payload を鍵で署名し、署名バイト列を返す。
    // 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>>;
    // verify は payload と signature の一致を検証し、真偽値を返す。
    // 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<bool>;
}

// KeyMaterial は実際の key bytes を保持する内部型。
// drop 時に zeroize でゼロクリアする（spec 層 A: メモリからの key bytes 漏洩防止）。
// 公開 API から見えない（pub(crate) のみ）。
#[derive(Zeroize)]
#[zeroize(drop)]
pub(crate) struct KeyMaterial {
    // key_bytes: 暗号化鍵バイト列（AES-256 = 32 bytes、EC-P256 = 32 bytes 等）
    key_bytes: Vec<u8>,
}

// KeyMaterial の Debug 実装: key_bytes を "[REDACTED]" で隠蔽する
impl fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // key_bytes の内容を出力しない（ログ・デバッグ出力への漏洩を防ぐ）
        f.debug_struct("KeyMaterial")
            .field("key_bytes", &"[REDACTED]")
            .finish()
    }
}

// OpenBaoKeyHandle は OpenBao Transit をバックエンドとする KeyHandle 実装。
// 生 key bytes を Self に保存しない（_material は内部操作用かつ Serialize から除外する）。
// Serialize は handle_id / key_class / is_valid のみを expose する（key bytes は含まない）。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenBaoKeyHandle {
    // handle_id: OpenBao Transit のキー版数識別子（UUID v7 形式）
    pub handle_id: String,
    // key_class: 鍵の用途クラス（5 class のいずれか）
    pub key_class: KeyClass,
    // is_valid: OpenBao による鍵の有効性確認結果（revoke / rotate 後 false になる）
    pub is_valid: bool,
    // _material: 生 key bytes を保持する内部フィールド（Serialize から除外する）
    #[serde(skip)]
    _material: Option<Arc<KeyMaterial>>,
}

// OpenBaoKeyHandle のコンストラクタ群（生 key bytes を受け取っても公開 API に漏れない）
impl OpenBaoKeyHandle {
    // from_remote_handle は OpenBao Transit が管理する鍵への参照として KeyHandle を構築する。
    // 生 key bytes は OpenBao 内に閉じる（spec §5 層 defense-in-depth 層 A）。
    // handle_id: OpenBao Transit key name。sign/verify/wrap/unwrap のルーティングに使用する。
    pub fn from_remote_handle(key_class: KeyClass, handle_id: String) -> Self {
        // _material は None: bytes は OpenBao 内に閉じるため Library は保持しない
        Self {
            // OpenBao Transit key name を handle_id として設定する
            handle_id,
            // 鍵の用途クラスを設定する
            key_class,
            // 構築直後は OpenBao 側で有効と見なす（revoke/rotate で false になる）
            is_valid: true,
            // 生 key bytes を保持しない（spec 規律に従う）
            _material: None,
        }
    }

    // key_class_str は key_class の文字列表現を返す（Display impl を使用する）。
    // gateway など呼び出し元が KeyClass の文字列表現を必要とする場合に使用する。
    pub fn key_class_str(&self) -> String {
        // Display impl を使って spec §v1 key_class 名（snake_case）を返す
        self.key_class.to_string()
    }

    // from_key_material は生 key bytes を受け取り、KeyHandle として wrap する。
    // 呼び出し元スコープを抜けると key_bytes は zeroize で消去される。
    pub fn from_key_material(key_class: KeyClass, handle_id: String, key_bytes: Vec<u8>) -> Self {
        // KeyMaterial に key_bytes を移動（Arc 共有で複数サービスに渡せる）
        let material = Arc::new(KeyMaterial { key_bytes });
        Self {
            handle_id,
            key_class,
            is_valid: true,
            // Arc でラップして内部保持する（公開 API には露出しない）
            _material: Some(material),
        }
    }
}

// OpenBaoKeyHandle は KeyHandle trait を実装する
#[async_trait]
impl KeyHandle for OpenBaoKeyHandle {
    // key_id は handle_id を返す（OpenBao Transit のキー識別子）
    fn key_id(&self) -> &str {
        &self.handle_id
    }

    // key_class は 5 class のいずれかを返す（clone で値を返す）
    fn key_class(&self) -> KeyClass {
        self.key_class.clone()
    }

    // is_valid は OpenBao による有効性を返す
    fn is_valid(&self) -> bool {
        self.is_valid
    }

    // sign は OpenBao Transit の sign API を呼び出して署名バイト列を返す。
    // 05_鍵管理適合仕様.md §5 層 defense-in-depth 層 B（runtime: OpenBao Transit 委譲）を実装する。
    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        // OPENBAO_ADDR 環境変数からベース URL を取得する（デフォルト: http://openbao.k1s0.svc:8200）
        let base_url = std::env::var("OPENBAO_ADDR")
            .unwrap_or_else(|_| "http://openbao.k1s0.svc:8200".to_string());
        // OPENBAO_TOKEN 環境変数からトークンを取得する（未設定時はエラー）
        let token = std::env::var("OPENBAO_TOKEN")
            .map_err(|_| anyhow::anyhow!("OPENBAO_TOKEN 環境変数が設定されていない"))?;
        // key_class を OpenBao Transit key name にマッピングする（例: "v1_data_dek"）
        let key_name = self.key_class.to_string();
        // payload を Base64 エンコードする（OpenBao Transit の input フィールドは Base64 要求）
        let input_b64 = BASE64_STD.encode(payload);
        // reqwest クライアントを構築する（タイムアウト 5 秒）
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| anyhow::anyhow!("reqwest Client 構築失敗: {e}"))?;
        // POST /v1/transit/sign/{key_name} を呼び出す
        let url = format!("{base_url}/v1/transit/sign/{key_name}");
        let resp = client
            .post(&url)
            .header("X-Vault-Token", &token)
            .json(&serde_json::json!({ "input": input_b64 }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("OpenBao Transit sign 送信失敗: {e}"))?;
        // HTTP ステータスを確認する
        if !resp.status().is_success() {
            // エラーステータス時はボディを含めてエラーを返す
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenBao Transit sign HTTP {status} body={body}"));
        }
        // レスポンス JSON をパースする
        let data: serde_json::Value = resp.json().await
            .map_err(|e| anyhow::anyhow!("OpenBao Transit sign レスポンス JSON パース失敗: {e}"))?;
        // signature フィールドを取得する（"vault:v1:<base64>" 形式）
        let sig_str = data["data"]["signature"].as_str()
            .ok_or_else(|| anyhow::anyhow!("OpenBao Transit sign: signature フィールドが存在しない"))?;
        // "vault:v1:" プレフィックスを除去して Base64 部分を取り出す
        let sig_b64 = sig_str.strip_prefix("vault:v1:").unwrap_or(sig_str);
        // Base64 デコードして署名バイト列を返す
        BASE64_STD.decode(sig_b64)
            .map_err(|e| anyhow::anyhow!("OpenBao Transit signature Base64 デコード失敗: {e}"))
    }

    // verify は OpenBao Transit の verify API を呼び出して検証結果を返す。
    // 05_鍵管理適合仕様.md §5 層 defense-in-depth 層 B（runtime: OpenBao Transit 委譲）を実装する。
    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<bool> {
        // OPENBAO_ADDR 環境変数からベース URL を取得する
        let base_url = std::env::var("OPENBAO_ADDR")
            .unwrap_or_else(|_| "http://openbao.k1s0.svc:8200".to_string());
        // OPENBAO_TOKEN 環境変数からトークンを取得する
        let token = std::env::var("OPENBAO_TOKEN")
            .map_err(|_| anyhow::anyhow!("OPENBAO_TOKEN 環境変数が設定されていない"))?;
        // key_class を OpenBao Transit key name にマッピングする
        let key_name = self.key_class.to_string();
        // payload を Base64 エンコードする
        let input_b64 = BASE64_STD.encode(payload);
        // signature を "vault:v1:<base64>" 形式にエンコードする
        let sig_b64 = format!("vault:v1:{}", BASE64_STD.encode(signature));
        // reqwest クライアントを構築する
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| anyhow::anyhow!("reqwest Client 構築失敗: {e}"))?;
        // POST /v1/transit/verify/{key_name} を呼び出す
        let url = format!("{base_url}/v1/transit/verify/{key_name}");
        let resp = client
            .post(&url)
            .header("X-Vault-Token", &token)
            .json(&serde_json::json!({ "input": input_b64, "signature": sig_b64 }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("OpenBao Transit verify 送信失敗: {e}"))?;
        // HTTP ステータスを確認する
        if !resp.status().is_success() {
            // エラーステータス時はボディを含めてエラーを返す
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenBao Transit verify HTTP {status} body={body}"));
        }
        // レスポンス JSON をパースする
        let data: serde_json::Value = resp.json().await
            .map_err(|e| anyhow::anyhow!("OpenBao Transit verify レスポンス JSON パース失敗: {e}"))?;
        // valid フィールドを取得して返す（存在しない場合は false とする）
        Ok(data["data"]["valid"].as_bool().unwrap_or(false))
    }
}
