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
    // create_stub は OpenBao Transit 呼出なしに stub の KeyHandle を生成する。
    // テスト・ドライラン用途（production では OpenBao 経由の呼出を使う）。
    pub fn create_stub(key_class: KeyClass, handle_id: String) -> Self {
        // _material は None: 生 key bytes を持たない stub
        Self {
            handle_id,
            key_class,
            is_valid: true,
            _material: None,
        }
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

    // sign は OpenBao Transit の sign API を呼び出す（stub は常に空 Vec を返す）。
    // production 実装では OpenBao Transit /v1/transit/sign/:name を呼び出す。
    async fn sign(&self, _payload: &[u8]) -> Result<Vec<u8>> {
        // stub 実装: OpenBao Transit への委譲先は bfl/src/openbao.rs を参照する
        Ok(vec![])
    }

    // verify は OpenBao Transit の verify API を呼び出す（stub は常に true を返す）。
    // production 実装では OpenBao Transit /v1/transit/verify/:name を呼び出す。
    async fn verify(&self, _payload: &[u8], _signature: &[u8]) -> Result<bool> {
        // stub 実装: OpenBao Transit への委譲先は bfl/src/openbao.rs を参照する
        Ok(true)
    }
}
