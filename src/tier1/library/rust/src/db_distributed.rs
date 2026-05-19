// db_distributed.rs — k1s0 tier1 Library: 分散 PostgreSQL 互換 DB の L1+ facade trait
// CockroachDB 等の分散 SQL を L1+ ラップする（OSS 型を公開 API に露出しない）。
// begin_serializable はシリアライザブル分離レベルでトランザクションを開始する。
// 分散 KV オペレーションはすべてトランザクション内で実行すること。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// crate::db::DbTransaction を再利用する（単一ノードとインターフェースを共有する）。
// 分散 DB でも同一の DbTransaction trait を通じて操作する（実装の差異を隠蔽する）。
use crate::db::DbTransaction;

// DistributedDb は分散 PostgreSQL 互換 DB の L1+ facade trait。
// CockroachDB / YugabyteDB 等の OSS 型を公開シグネチャに露出しない。
// 分散トランザクションのみをサポートし、単一ノード操作は crate::db::Db を使用する。
#[async_trait]
pub trait DistributedDb: Send + Sync {
    // begin_serializable はシリアライザブル分離レベルで分散トランザクションを開始する。
    // 分散 KV オペレーションのトランザクションに使用する（READ COMMITTED 不可）。
    // 実装は CockroachDB の SERIALIZABLE 分離レベルを保証する必要がある。
    async fn begin_serializable(&self) -> crate::Result<Box<dyn DbTransaction>>;
}
