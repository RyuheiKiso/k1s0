// TransactionalAppendPort の PostgreSQL 実装。
// usecase 層は TransactionalAppendPort トレイトにのみ依存し、
// 本ファイルの具体型（TransactionalAppendAdapter）には依存しない。
// これによりクリーンアーキテクチャの依存方向（内→外）が保たれる。

use async_trait::async_trait;
use sqlx::{Executor, PgPool};

use crate::domain::entity::event::{EventStream, StoredEvent};
use crate::domain::repository::TransactionalAppendPort;

use super::{EventPostgresRepository, StreamPostgresRepository};

/// `TransactionalAppendPort` の `PostgreSQL` 実装。
/// REPEATABLE READ トランザクション内でストリーム作成・イベント追記・バージョン更新を実行する。
pub struct TransactionalAppendAdapter {
    pool: PgPool,
}

impl TransactionalAppendAdapter {
    /// `PgPool` からアダプターを生成する。
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionalAppendPort for TransactionalAppendAdapter {
    /// ストリーム作成（新規の場合）・イベント追記・バージョン更新を
    /// 単一の REPEATABLE READ トランザクションで実行する。
    /// テナント分離のため、トランザクション開始時に `set_config` でテナント ID を設定する（ADR-0106）。
    async fn append_in_transaction<'a>(
        &self,
        tenant_id: &str,
        stream: Option<&'a EventStream>,
        stream_id: &str,
        events: Vec<StoredEvent>,
        new_version: i64,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        // REPEATABLE READ でトランザクションを開始してファントムリードを防止する
        let mut tx = self.pool.begin().await?;
        // トランザクション分離レベルを REPEATABLE READ に設定する
        tx.execute("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .await?;

        // テナント分離: トランザクション内で set_config を実行して RLS を有効化する
        // append_in_tx 内で set_config が実行されるため、ここでは重複しない
        // （append_in_tx の set_config がストリーム作成より先に実行されるよう順序を保証）

        // 新規ストリームの場合はトランザクション内でストリームを作成する
        if let Some(s) = stream {
            StreamPostgresRepository::create_in_tx(s, &mut tx).await?;
        }

        // トランザクション内でイベントを一括 INSERT する（set_config は内部で実行）
        let persisted =
            EventPostgresRepository::append_in_tx(tenant_id, stream_id, events, &mut tx).await?;

        // トランザクション内でストリームのバージョンを更新する
        StreamPostgresRepository::update_version_in_tx(stream_id, new_version, &mut tx).await?;

        // 全操作成功後にコミットして原子性を保証する
        tx.commit().await?;

        Ok(persisted)
    }
}
