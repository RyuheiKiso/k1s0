//! トランザクション境界
//!
//! application 層から扱えるトランザクション境界を提供する。
//! 実際のDB実装は別途行うが、ここではインターフェースを定義する。

use std::future::Future;

use crate::error::DbResult;

/// トランザクションの分離レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read Uncommitted
    ReadUncommitted,
    /// Read Committed（デフォルト）
    ReadCommitted,
    /// Repeatable Read
    RepeatableRead,
    /// Serializable
    Serializable,
}

impl IsolationLevel {
    /// SQL 文字列表現を取得
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::ReadUncommitted => "READ UNCOMMITTED",
            Self::ReadCommitted => "READ COMMITTED",
            Self::RepeatableRead => "REPEATABLE READ",
            Self::Serializable => "SERIALIZABLE",
        }
    }
}

impl Default for IsolationLevel {
    fn default() -> Self {
        Self::ReadCommitted
    }
}

/// トランザクションモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    /// 読み書き（デフォルト）
    ReadWrite,
    /// 読み取り専用
    ReadOnly,
}

impl TransactionMode {
    /// SQL 文字列表現を取得
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::ReadWrite => "READ WRITE",
            Self::ReadOnly => "READ ONLY",
        }
    }
}

impl Default for TransactionMode {
    fn default() -> Self {
        Self::ReadWrite
    }
}

/// トランザクションオプション
#[derive(Debug, Clone, Default)]
pub struct TransactionOptions {
    /// 分離レベル
    pub isolation_level: IsolationLevel,
    /// トランザクションモード
    pub mode: TransactionMode,
}

impl TransactionOptions {
    /// 新しいオプションを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 分離レベルを設定
    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = level;
        self
    }

    /// トランザクションモードを設定
    pub fn with_mode(mut self, mode: TransactionMode) -> Self {
        self.mode = mode;
        self
    }

    /// 読み取り専用トランザクションを作成
    pub fn read_only() -> Self {
        Self::new().with_mode(TransactionMode::ReadOnly)
    }

    /// Serializable トランザクションを作成
    pub fn serializable() -> Self {
        Self::new().with_isolation_level(IsolationLevel::Serializable)
    }
}

/// トランザクション実行トレイト
///
/// application 層でトランザクションを扱うためのインターフェース。
/// 具体的な実装は infrastructure 層で行う。
pub trait TransactionExecutor: Send + Sync {
    /// トランザクション内で処理を実行
    ///
    /// # Arguments
    ///
    /// * `f` - トランザクション内で実行する関数
    ///
    /// # Returns
    ///
    /// 関数の戻り値、またはトランザクションエラー
    fn execute<F, Fut, T>(&self, f: F) -> impl Future<Output = DbResult<T>> + Send
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = DbResult<T>> + Send,
        T: Send;

    /// オプション付きでトランザクションを実行
    fn execute_with_options<F, Fut, T>(
        &self,
        options: TransactionOptions,
        f: F,
    ) -> impl Future<Output = DbResult<T>> + Send
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = DbResult<T>> + Send,
        T: Send;
}

/// 単位作業（Unit of Work）パターン
///
/// 複数のリポジトリ操作をトランザクションで囲む。
pub trait UnitOfWork: Send + Sync {
    /// トランザクションを開始
    fn begin(&self) -> impl Future<Output = DbResult<()>> + Send;

    /// トランザクションをコミット
    fn commit(&self) -> impl Future<Output = DbResult<()>> + Send;

    /// トランザクションをロールバック
    fn rollback(&self) -> impl Future<Output = DbResult<()>> + Send;

    /// トランザクション内で処理を実行
    fn run<F, Fut, T>(&self, f: F) -> impl Future<Output = DbResult<T>> + Send
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = DbResult<T>> + Send,
        T: Send;
}

/// トランザクション状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// 未開始
    NotStarted,
    /// 進行中
    Active,
    /// コミット済み
    Committed,
    /// ロールバック済み
    RolledBack,
}

impl TransactionState {
    /// アクティブかどうか
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// 完了しているかどうか
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Committed | Self::RolledBack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level() {
        assert_eq!(IsolationLevel::ReadCommitted.as_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::Serializable.as_sql(), "SERIALIZABLE");
        assert_eq!(IsolationLevel::default(), IsolationLevel::ReadCommitted);
    }

    #[test]
    fn test_transaction_mode() {
        assert_eq!(TransactionMode::ReadWrite.as_sql(), "READ WRITE");
        assert_eq!(TransactionMode::ReadOnly.as_sql(), "READ ONLY");
        assert_eq!(TransactionMode::default(), TransactionMode::ReadWrite);
    }

    #[test]
    fn test_transaction_options() {
        let options = TransactionOptions::new()
            .with_isolation_level(IsolationLevel::Serializable)
            .with_mode(TransactionMode::ReadOnly);

        assert_eq!(options.isolation_level, IsolationLevel::Serializable);
        assert_eq!(options.mode, TransactionMode::ReadOnly);
    }

    #[test]
    fn test_transaction_options_read_only() {
        let options = TransactionOptions::read_only();
        assert_eq!(options.mode, TransactionMode::ReadOnly);
    }

    #[test]
    fn test_transaction_options_serializable() {
        let options = TransactionOptions::serializable();
        assert_eq!(options.isolation_level, IsolationLevel::Serializable);
    }

    #[test]
    fn test_transaction_state() {
        assert!(!TransactionState::NotStarted.is_active());
        assert!(TransactionState::Active.is_active());
        assert!(!TransactionState::Committed.is_active());

        assert!(!TransactionState::Active.is_finished());
        assert!(TransactionState::Committed.is_finished());
        assert!(TransactionState::RolledBack.is_finished());
    }
}
