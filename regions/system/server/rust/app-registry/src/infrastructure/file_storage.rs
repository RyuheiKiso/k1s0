use std::path::{Path, PathBuf};

/// FileStorage はローカルファイルシステム上のアプリバイナリを管理する。
/// S3の代替として、PV（Persistent Volume）マウント先のディレクトリから
/// storage_key に対応するファイルを読み取る。
pub struct FileStorage {
    /// アプリバイナリを格納するルートディレクトリ。
    root_path: PathBuf,
}

impl FileStorage {
    /// 指定されたルートパスで FileStorage を作成する。
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
        }
    }

    /// storage_key をファイルシステムのフルパスに変換する。
    /// パストラバーサル攻撃を防ぐため、root_path 外へのパスは拒否する。
    pub fn resolve_path(&self, storage_key: &str) -> anyhow::Result<PathBuf> {
        let key_path = Path::new(storage_key);
        if key_path.is_absolute() || key_path.components().any(|c| {
            matches!(c, std::path::Component::ParentDir | std::path::Component::Prefix(_))
        }) {
            anyhow::bail!("不正なストレージキー: {}", storage_key);
        }
        Ok(self.root_path.join(key_path))
    }

    /// 指定された storage_key に対応するファイルのバイト列を読み取る。
    pub async fn read_file(&self, storage_key: &str) -> anyhow::Result<Vec<u8>> {
        let path = self.resolve_path(storage_key)?;
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| anyhow::anyhow!("ファイルの読み取りに失敗: {} ({})", storage_key, e))?;
        Ok(bytes)
    }

    /// 指定された storage_key に対応するファイルが存在するか確認する。
    pub fn file_exists(&self, storage_key: &str) -> bool {
        self.resolve_path(storage_key)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// ファイルのバイト単位のサイズを返す。ファイルが存在しない場合は None を返す。
    pub async fn file_size(&self, storage_key: &str) -> Option<u64> {
        let path = self.resolve_path(storage_key).ok()?;
        tokio::fs::metadata(&path).await.ok().map(|m| m.len())
    }
}
