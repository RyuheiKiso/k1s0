// secret.rs — k1s0 tier1 Library: シークレット管理 L1+ facade trait
// OpenBao 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// 生シークレット値を公開 API に露出禁止（SecretHandle による opaque ラップを必須とする）。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// SecretStore は OpenBao を L1+ ラップするシークレット管理 facade trait。
// 公開 API シグネチャに OSS 型（vaultrs::client::VaultClient 等）を一切含まない。
// 取得したシークレット値は SecretHandle にラップして返す（生 bytes 露出禁止）。
#[async_trait]
pub trait SecretStore: Send + Sync {
    // get_secret はシークレット名で値を取得して SecretHandle にラップして返す。
    // name はシークレット識別子（例: "db/password" / "jwt/signing_key"）。
    // 戻り値は SecretHandle（生シークレット bytes は公開 API に露出しない）。
    async fn get_secret(&self, name: &str) -> crate::Result<SecretHandle>;

    // put_secret はシークレットを設定する（値は SecretHandle で渡す）。
    // name はシークレット識別子（例: "db/password"）。
    // secret は SecretHandle（生 bytes を直接受け取らない設計で規律を強制する）。
    async fn put_secret(&self, name: &str, secret: SecretHandle) -> crate::Result<()>;
}

// SecretHandle は生シークレット値を保持しない opaque ハンドル型。
// pub(crate) のみで内部アクセスを許可し、公開 API への生 bytes 露出を禁止する。
// drop 時のゼロクリアは実装の KeyMaterial と同様のパターンで対応すること。
pub struct SecretHandle(Vec<u8>);

// SecretHandle のコンストラクタと内部アクセサ（pub(crate) で公開 API 露出禁止）
impl SecretHandle {
    // from_raw は内部（opaque）値を生成する — 実装内部のみ使用可。
    // pub(crate) 限定のため、外部クレートから生 bytes を直接 SecretHandle に変換できない。
    pub(crate) fn from_raw(raw: Vec<u8>) -> Self {
        // Vec<u8> をそのまま保持する（opaque ラップ）
        Self(raw)
    }

    // as_bytes は内部値への参照を返す — pub(crate) のみ許可（公開 API への露出禁止）。
    // 外部クレートからは生 bytes にアクセスできない（ゼロ知識原則の強制）。
    pub(crate) fn as_bytes(&self) -> &[u8] {
        // 内部バイト列への参照を返す
        &self.0
    }
}
