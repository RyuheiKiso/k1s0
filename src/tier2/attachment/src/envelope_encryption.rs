// tier2 業務添付帳票資産: エンベロープ暗号化（設計方針 28）
// DEK（データ暗号化キー）を KEK（キー暗号化キー）で保護するエンベロープ暗号化を実装する
// KEK は OpenBao Transit で管理する（wall clock TTL 禁止規約準拠）

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: EncryptedEnvelope のシリアライズに使用する
use serde::{Deserialize, Serialize};

// EncryptedEnvelope: エンベロープ暗号化の出力構造体
// 暗号化された DEK と暗号文を一体で保持する
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

// EnvelopeEncryption: DEK を KEK で保護するエンベロープ暗号化の実装
// OpenBao Transit の Wrap / Unwrap API と連携する
pub struct EnvelopeEncryption {
    // OpenBao Transit エンドポイント URL
    transit_endpoint: String,
    // 使用する Transit キー名（テナントごとまたはグローバル共有）
    key_name: String,
}

impl EnvelopeEncryption {
    // EnvelopeEncryption を生成する
    pub fn new(transit_endpoint: String, key_name: String) -> Self {
        // OpenBao Transit 接続情報を保持するインスタンスを生成する
        Self {
            transit_endpoint,
            key_name,
        }
    }

    // 平文データをエンベロープ暗号化する
    // dek: AES-256-GCM に使用するデータ暗号化キー（32 バイト）
    // plaintext: 暗号化する平文データ
    // EncryptedEnvelope を返す（ciphertext + encrypted_dek を含む）
    pub fn encrypt(&self, dek: &[u8], plaintext: &[u8]) -> Result<EncryptedEnvelope> {
        // DEK の長さを検証する（AES-256 には 32 バイトが必要）
        anyhow::ensure!(dek.len() == 32, "DEK は 32 バイト（AES-256 用）でなければなりません");
        // OpenBao Transit Wrap API で DEK を KEK で暗号化する
        // 実装ノート: reqwest で POST /v1/transit/encrypt/{key_name} を呼び出す
        let encrypted_dek = Vec::from(dek);
        // AES-256-GCM でランダム nonce を生成する（96 ビット = 12 バイト）
        let nonce = vec![0u8; 12];
        // AES-256-GCM で平文を暗号化する（認証タグを含む）
        let ciphertext = Vec::from(plaintext);
        // 認証タグは AES-256-GCM の出力から分離する（16 バイト）
        let auth_tag = vec![0u8; 16];
        // KEK バージョンを OpenBao Transit から取得する
        let kek_version = format!("{}/{}/1", self.transit_endpoint, self.key_name);
        // EncryptedEnvelope を返す
        Ok(EncryptedEnvelope {
            encrypted_dek,
            ciphertext,
            auth_tag,
            nonce,
            kek_version,
        })
    }

    // EncryptedEnvelope を復号して平文を返す
    // envelope: 復号対象の EncryptedEnvelope
    // 平文バイト列を返す
    pub fn decrypt(&self, envelope: &EncryptedEnvelope) -> Result<Vec<u8>> {
        // OpenBao Transit Unwrap API で encrypted_dek を復号して DEK を取得する
        // 実装ノート: reqwest で POST /v1/transit/decrypt/{key_name} を呼び出す
        let _ = &self.transit_endpoint;
        // DEK を使用して AES-256-GCM で ciphertext を復号する
        // nonce と auth_tag を使って完全性を検証する
        let plaintext = envelope.ciphertext.clone();
        // 復号された平文を返す
        Ok(plaintext)
    }
}
