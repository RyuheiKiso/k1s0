// secret.rs — k1s0 tier1 Library core: Secret Management L3 facade
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）に準拠する。
// key_handle.rs の KeyClass / KeyHandle / OpenBaoKeyHandle をこのモジュールから re-export する。
// 追加で SecretStore trait を定義する（シークレット取得抽象）。
// 公開 API に生 key bytes / 生 secret 値を露出しない。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: SecretMetadata のシリアライズに使用する
use serde::{Deserialize, Serialize};
// zeroize: SecretValue のゼロクリアに使用する
use zeroize::Zeroize;
// std::fmt: SecretValue の Debug 実装に使用する
use std::fmt;

// key_handle.rs から KeyClass / KeyHandle / OpenBaoKeyHandle を re-export する
// （後方互換のため crate::key_handle も維持する）
// KeyMaterial は pub(crate) のため再エクスポートしない（生 key bytes 隠蔽を維持する）
pub use crate::key_handle::{KeyClass, KeyHandle, OpenBaoKeyHandle};

// SecretMetadata はシークレットのメタデータを表す Library 独自型。
// 生のシークレット値は含まない（SecretValue 型で別管理する）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    // secret_id: シークレットの識別子（UUID v7 形式）
    pub secret_id: String,
    // version: シークレットのバージョン番号（ローテーション追跡用）
    pub version: u32,
    // key_class: このシークレットを保護している鍵のクラス
    pub key_class: KeyClass,
    // tenant_id: シークレットが属するテナントの識別子
    pub tenant_id: String,
    // is_active: シークレットが有効かどうか（revoke / rotate 後 false になる）
    pub is_active: bool,
}

// SecretValue はシークレットの実値を保持する内部型。
// Drop 時に zeroize でゼロクリアする（メモリからの漏洩を防ぐ）。
// 公開 API には返さない（SecretStore::get 内で消費する）。
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecretValue {
    // value_bytes: シークレットの生バイト列（パスワード / API key 等）
    value_bytes: Vec<u8>,
}

// SecretValue の Debug 実装: value_bytes を "[REDACTED]" で隠蔽する
impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // value_bytes の内容を出力しない（ログ・デバッグ出力への漏洩を防ぐ）
        f.debug_struct("SecretValue")
            .field("value_bytes", &"[REDACTED]")
            .finish()
    }
}

// SecretValue のコンストラクタと操作メソッド
impl SecretValue {
    // new は生バイト列を受け取り SecretValue を生成する（内部使用のみ）
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        // 受け取ったバイト列を保持する
        Self { value_bytes: bytes }
    }

    // len はバイト列の長さを返す（値自体を返さない）
    pub fn len(&self) -> usize {
        // バイト列の長さのみ公開する
        self.value_bytes.len()
    }

    // is_empty はバイト列が空かどうかを返す
    pub fn is_empty(&self) -> bool {
        // バイト列が空かどうかを返す
        self.value_bytes.is_empty()
    }

    // with_bytes はクロージャにバイト列を渡して処理させる。
    // クロージャ外にバイト列が漏れないようにするための設計。
    pub fn with_bytes<F, T>(&self, f: F) -> T
    where
        // クロージャはバイト列を受け取り任意の T を返す
        F: FnOnce(&[u8]) -> T,
    {
        // バイト列をクロージャに渡す（所有権を移さない）
        f(&self.value_bytes)
    }
}

// SecretRotationPolicy はシークレットローテーションポリシーを表す enum。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretRotationPolicy {
    // Manual: 手動ローテーション（自動ローテーションなし）
    Manual,
    // OnDemand: 要求時ローテーション（呼び出し元が rotate() を呼ぶ）
    OnDemand,
    // Scheduled: スケジュールローテーション（間隔 HLC tick 数で指定）
    Scheduled { interval_ticks: u64 },
}

// SecretStore は Secret Management の L3 抽象 trait。
// OpenBao / AWS Secrets Manager 等の OSS/サービスを実装で切り替えられる。
// 公開 API に生シークレット値を返さない（with_bytes パターンを使う）。
#[async_trait]
pub trait SecretStore: Send + Sync {
    // get_metadata はシークレットのメタデータのみを返す（値は返さない）。
    // メタデータは SecretMetadata 型で返す（OSS 固有型は露出しない）。
    async fn get_metadata(&self, secret_id: &str, tenant_id: &str)
        -> Result<Option<SecretMetadata>>;

    // with_secret はシークレット値をクロージャに渡して処理させる。
    // クロージャ外にシークレット値が漏れない設計（値は返さない）。
    async fn with_secret<F, T>(
        &self,
        secret_id: &str,
        tenant_id: &str,
        f: F,
    ) -> Result<T>
    where
        // クロージャはシークレット値を受け取り T を返す
        F: FnOnce(&SecretValue) -> T + Send;

    // rotate はシークレットを新しいバージョンにローテーションする。
    // 返した SecretMetadata に新しい version が含まれる。
    async fn rotate(&self, secret_id: &str, tenant_id: &str) -> Result<SecretMetadata>;

    // revoke はシークレットを無効化する（is_active=false にする）。
    // revoke 後の with_secret 呼び出しはエラーを返す。
    async fn revoke(&self, secret_id: &str, tenant_id: &str) -> Result<()>;
}
