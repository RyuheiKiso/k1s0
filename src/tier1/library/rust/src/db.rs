// db.rs — k1s0 tier1 Library: 単一ノード PostgreSQL 操作の L1+ facade trait
// sqlx 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// tenant_id は GUC 経由で注入済みであること（Repository trait 経由での使用が必須）。
// 生 SQL 文字列受付 API は提供しない（04_認証適合仕様.md §CI 不変条件 整合 5 準拠）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// ToSql は dyn trait として受け取るためのトレイト境界（PostgreSQL パラメータバインド用）。
// OSS 型（sqlx::Type 等）を公開 API に含めないため、標準的な文字列変換で対応する。
// 注意: 本 trait は下記 DbTransaction::execute / query で &[&(dyn ToSql + Sync)] として受け取る。
// 実装側は適切な型エラスチャー（Box<dyn std::error::Error>）で対応する。
pub trait ToSql: std::fmt::Debug {}

// Db は単一ノード PostgreSQL 操作の L1+ facade trait。
// sqlx PgPool 等の OSS 型を公開シグネチャに露出しない。
// tenant_id は GUC 経由で注入済みであること（SET LOCAL app.tenant_id を事前実行する）。
#[async_trait]
pub trait Db: Send + Sync {
    // begin はロールバック可能なトランザクションを開始して Box<dyn DbTransaction> を返す。
    // トランザクション内で tenant_id GUC が有効であることを保証する実装が必要。
    async fn begin(&self) -> crate::Result<Box<dyn DbTransaction>>;
}

// DbTransaction は単一トランザクション内の SQL 操作を提供する trait。
// トランザクションは commit または rollback で終了する（いずれかを必ず呼ぶこと）。
#[async_trait]
pub trait DbTransaction: Send + Sync {
    // execute はクエリを実行して影響を受けた行数を返す。
    // params は &[&(dyn ToSql + Sync)] 形式で渡す（型消去によりコンパイル時型安全を維持する）。
    async fn execute(&mut self, query: &str, params: &[&(dyn ToSql + Sync)]) -> crate::Result<u64>;

    // query はクエリを実行して行の Vec を返す（各行は Vec<String> で表現する）。
    // 実装側で適切な型変換（JSON 経由等）を行うこと。
    async fn query(&mut self, query: &str, params: &[&(dyn ToSql + Sync)]) -> crate::Result<Vec<Vec<String>>>;

    // commit はトランザクションをコミットする（Box<Self> を消費して所有権を手放す）。
    // commit / rollback のいずれかを必ず呼ぶこと（呼ばない場合はトランザクションが未完了になる）。
    async fn commit(self: Box<Self>) -> crate::Result<()>;

    // rollback はトランザクションをロールバックする（Box<Self> を消費して所有権を手放す）。
    // エラー発生時は rollback を明示的に呼ぶこと（Drop 時の暗黙ロールバックに依存しない）。
    async fn rollback(self: Box<Self>) -> crate::Result<()>;
}
