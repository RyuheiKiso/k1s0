// TransactionalAppendPort の PostgreSQL 実装。
// usecase 層は TransactionalAppendPort トレイトにのみ依存し、
// 本ファイルの具体型（TransactionalAppendAdapter）には依存しない。
// これによりクリーンアーキテクチャの依存方向（内→外）が保たれる。

use async_trait::async_trait;
use sqlx::{Executor, PgPool};

use crate::domain::entity::event::{EventStream, StoredEvent};
use crate::domain::repository::TransactionalAppendPort;

use super::{EventPostgresRepository, StreamPostgresRepository};

/// TransactionalAppendPort の PostgreSQL 実装。
/// REPEATABLE READ トランザクション内でストリーム作成・イベント追記・バージョン更新を実行する。
pub struct TransactionalAppendAdapter {
    pool: PgPool,
}

impl TransactionalAppendAdapter {
    /// PgPool からアダプターを生成する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionalAppendPort for TransactionalAppendAdapter {
    /// ストリーム作成（新規の場合）・イベント追記・バージョン更新を
    /// 単一の REPEATABLE READ トランザクションで実行する。
    async fn append_in_transaction(
        &self,
        stream: Option<&EventStream>,
        stream_id: &str,
        events: Vec<StoredEvent>,
        new_version: i64,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        // REPEATABLE READ でトランザクションを開始してファントムリードを防止する
        let mut tx = self.pool.begin().await?;
        // トランザクション分離レベルを REPEATABLE READ に設定する
        tx.execute("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .await?;

        // 新規ストリームの場合はトランザクション内でストリームを作成する
        if let Some(s) = stream {
            StreamPostgresRepository::create_in_tx(s, &mut tx).await?;
        }

        // トランザクション内でイベントを一括 INSERT する
        let persisted =
            EventPostgresRepository::append_in_tx(stream_id, events, &mut tx).await?;

        // トランザクション内でストリームのバージョンを更新する
        StreamPostgresRepository::update_version_in_tx(stream_id, new_version, &mut tx).await?;

        // 全操作成功後にコミットして原子性を保証する
        tx.commit().await?;

        Ok(persisted)
    }
}
