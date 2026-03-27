// トランザクション型イベント追記のドメインポートトレイト。
// クリーンアーキテクチャの原則により、usecase 層は infrastructure 具体型
//（EventPostgresRepository, StreamPostgresRepository）に直接依存してはならない。
// このトレイトはトランザクション内でのストリーム作成・イベント追記・バージョン更新を
// 単一の原子操作として抽象化する。

use async_trait::async_trait;

use crate::domain::entity::event::{EventStream, StoredEvent};

/// トランザクション型イベント追記のドメインポート。
/// 実装は infrastructure 層に置き、usecase は本トレイトを通じてのみ操作する。
/// これにより usecase 層から sqlx や PostgreSQL 等の具体型依存を排除する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TransactionalAppendPort: Send + Sync {
    /// 新規ストリームの場合はストリームを作成し、イベントを追記してバージョンを更新する。
    /// 既存ストリームの場合はイベントを追記してバージョンを更新する。
    /// 全操作は単一のデータベーストランザクション（REPEATABLE READ）内で実行される。
    ///
    /// # Arguments
    /// - `stream` - 新規作成する場合のストリーム（None なら既存ストリームへの追記）
    /// - `stream_id` - 追記対象のストリーム ID
    /// - `events` - 追記するイベント群（バージョンは呼び出し元が設定済み）
    /// - `new_version` - 追記後のストリームバージョン
    async fn append_in_transaction(
        &self,
        stream: Option<&EventStream>,
        stream_id: &str,
        events: Vec<StoredEvent>,
        new_version: i64,
    ) -> anyhow::Result<Vec<StoredEvent>>;
}
