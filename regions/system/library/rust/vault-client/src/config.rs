use std::time::Duration;

#[derive(Debug, Clone)]
pub struct VaultClientConfig {
    pub server_url: String,
    pub cache_ttl: Duration,
    pub cache_max_capacity: usize,
    /// Vault 接続が必須かどうかを制御するフラグ。
    ///
    /// - `true` (デフォルト): Vault 接続失敗時にエラーを返す。本番環境でシークレットの
    ///   取得が必須の場合に使用する。シークレットなしでの起動はセキュリティリスクとなるため、
    ///   本番環境では必ず `true` にすること。
    /// - `false`: Vault 接続失敗時に警告ログを出力し、ローカル設定値で続行する。
    ///   開発環境やテスト環境で Vault が利用できない場合のフォールバックとして使用する。
    pub vault_required: bool,
}

impl Default for VaultClientConfig {
    fn default() -> Self {
        Self::new("http://localhost:8080")
    }
}

impl VaultClientConfig {
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            cache_ttl: Duration::from_secs(600),
            cache_max_capacity: 500,
            // 本番安全性のためデフォルトは true（Vault 必須）
            vault_required: true,
        }
    }

    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    pub fn cache_max_capacity(mut self, capacity: usize) -> Self {
        self.cache_max_capacity = capacity;
        self
    }

    /// Vault 接続の必須/任意を設定するビルダーメソッド。
    /// 開発環境では `false` に設定することで、Vault が利用不可でもサーバーを起動できる。
    pub fn vault_required(mut self, required: bool) -> Self {
        self.vault_required = required;
        self
    }
}
