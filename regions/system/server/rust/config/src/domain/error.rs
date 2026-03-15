use std::fmt;

/// ConfigRepositoryError はリポジトリ層のエラーを型安全に表す。
/// 文字列マッチングによるエラー分類を排除するための型付きエラー。
#[derive(Debug)]
pub enum ConfigRepositoryError {
    /// 指定された namespace/key の設定値が見つからない
    NotFound { namespace: String, key: String },
    /// 楽観的排他制御によるバージョン不一致
    VersionConflict { expected: i32, current: i32 },
    /// 指定されたサービス名に紐づく設定が見つからない
    ServiceNotFound(String),
    /// DB接続エラー等のインフラストラクチャエラー
    Infrastructure(anyhow::Error),
}

/// ConfigRepositoryError の表示フォーマット実装
impl fmt::Display for ConfigRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { namespace, key } => {
                write!(f, "config not found: {}/{}", namespace, key)
            }
            Self::VersionConflict { expected, current } => {
                write!(
                    f,
                    "version conflict: expected={}, current={}",
                    expected, current
                )
            }
            Self::ServiceNotFound(name) => {
                write!(f, "service not found: {}", name)
            }
            Self::Infrastructure(e) => {
                write!(f, "infrastructure error: {}", e)
            }
        }
    }
}

/// std::error::Error トレイト実装（エラーチェーンのサポート）
impl std::error::Error for ConfigRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Infrastructure(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}
