use std::collections::HashMap;

use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

/// HashiCorp Vault KV v2 クライアント。
pub struct VaultKvClient {
    client: VaultClient,
    mount: String,
}

impl VaultKvClient {
    /// 新しい Vault KV v2 クライアントを作成する。
    pub fn new(addr: &str, token: &str) -> anyhow::Result<Self> {
        let client = VaultClient::new(
            VaultClientSettingsBuilder::default()
                .address(addr)
                .token(token)
                .build()?,
        )?;
        Ok(Self {
            client,
            mount: "secret".to_string(),
        })
    }

    /// KV マウントパスを設定する。
    pub fn with_mount(mut self, mount: impl Into<String>) -> Self {
        self.mount = mount.into();
        self
    }

    /// 指定パスのシークレットを読み取る。
    pub async fn read_secret(&self, path: &str) -> anyhow::Result<HashMap<String, String>> {
        let data: HashMap<String, String> =
            vaultrs::kv2::read(&self.client, &self.mount, path).await?;
        Ok(data)
    }

    /// 指定パスにシークレットを書き込む。
    pub async fn write_secret(
        &self,
        path: &str,
        data: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        vaultrs::kv2::set(&self.client, &self.mount, path, data).await?;
        Ok(())
    }

    /// 指定パスのシークレットの最新バージョンを削除する。
    pub async fn delete_secret(&self, path: &str) -> anyhow::Result<()> {
        vaultrs::kv2::delete_latest(&self.client, &self.mount, path).await?;
        Ok(())
    }

    /// 指定パス配下のシークレット一覧を取得する。
    pub async fn list_secrets(&self, path: &str) -> anyhow::Result<Vec<String>> {
        let keys = vaultrs::kv2::list(&self.client, &self.mount, path).await?;
        Ok(keys)
    }
}
