// key_handle.rs — spec 05 鍵管理適合仕様: KeyHandle opaque 型
// 04_認証適合仕様.md §v1 auth_class と 05_鍵管理適合仕様.md §v1 key_class に基づく。
// 公開 API シグネチャに生 key bytes を露出しない opaque 型を実装する。
// KeyMaterial は zeroize で drop 時にメモリをゼロクリアする。

use std::fmt;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

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
    _material: Option<std::sync::Arc<KeyMaterial>>,
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
        // KeyMaterial に key_bytes を移動（arc 共有で複数サービスに渡せる）
        let material = std::sync::Arc::new(KeyMaterial { key_bytes });
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
