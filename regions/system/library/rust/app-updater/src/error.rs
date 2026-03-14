/// アプリアップデーター操作中に発生するエラー
#[derive(Debug, thiserror::Error)]
pub enum AppUpdaterError {
    /// サーバーへの接続エラー
    #[error("connection error: {0}")]
    Connection(String),
    /// 設定値が不正
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    /// レスポンスのパースエラー
    #[error("parse error: {0}")]
    Parse(String),
    /// 認証エラー（401）
    #[error("unauthorized")]
    Unauthorized,
    /// 指定したアプリが見つからない（404）
    #[error("app not found: {0}")]
    AppNotFound(String),
    /// 指定したバージョンが見つからない
    #[error("version not found: {0}")]
    VersionNotFound(String),
    /// チェックサム不一致
    #[error("checksum mismatch: {0}")]
    Checksum(String),
    /// I/O エラー
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
