// db.rs — k1s0 tier1 Library backend: Relational Store/Single-leader L1+ facade
// repository.rs の Repository<T> / PgRepository<T> をこのモジュールから re-export する。
// L1+: OSS の全機能を表現 + tier1 横断要素（auth context 伝播 / retry / tracing）を強制。
// 追加で TransactionalRepository<T> trait を定義する（明示的 transaction 管理）。
// 公開 API に OSS 型（sqlx::PgPool / sqlx::Transaction 等）を露出しない。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: エンティティのシリアライズ制約に使用する
use serde::{de::DeserializeOwned, Serialize};
// AuthContext: DB 操作に認証コンテキストを伝播する
use crate::core::auth::AuthContext;
// SpanContext: DB 操作に tracing コンテキストを伝播する
use crate::core::observability::SpanContext;

// repository.rs から Repository<T> / PgRepository<T> を re-export する
// （後方互換のため crate::repository も維持する）
pub use crate::repository::{PgRepository, Repository};

// QueryPage はページネーション付きクエリの結果を表す Library 独自型。
// OSS の Cursor / PaginationToken を Library 独自型に変換して露出しない。
#[derive(Debug, Clone)]
pub struct QueryPage<T> {
    // items: 取得したエンティティのリスト
    pub items: Vec<T>,
    // next_cursor: 次ページのカーソル（None は最終ページ）
    pub next_cursor: Option<String>,
    // total_count: フィルター条件に一致する総件数（不明の場合は None）
    pub total_count: Option<u64>,
}

// QueryFilter はクエリのフィルター条件を表す Library 独自型。
// 生 SQL WHERE 句文字列を受け取らない（SQL injection 防止）。
#[derive(Debug, Clone)]
pub struct QueryFilter {
    // column: フィルター対象カラム名（allowlist で制限する実装を推奨する）
    pub column: String,
    // operator: フィルター演算子（Library 独自語彙: Eq / Neq / Gt / Lt / Like）
    pub operator: FilterOperator,
    // value: フィルター値（JSON 値として渡す; bind パラメータとして使用する）
    pub value: serde_json::Value,
}

// FilterOperator はフィルター演算子を表す Library 独自型。
// 生 SQL 演算子文字列を受け取らない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterOperator {
    // Eq: 等値（= $1）
    Eq,
    // Neq: 非等値（<> $1）
    Neq,
    // Gt: 大なり（> $1）
    Gt,
    // Lt: 小なり（< $1）
    Lt,
    // Gte: 大なりイコール（>= $1）
    Gte,
    // Lte: 小なりイコール（<= $1）
    Lte,
    // Like: LIKE パターンマッチ（LIKE $1）
    Like,
    // In: IN リスト（IN ($1, $2, ...)）
    In,
}

// RepositoryQuery はページネーション付きクエリのパラメータを表す struct。
// 生 SQL 文字列を受け取らない設計（SQL injection 防止）。
#[derive(Debug, Clone)]
pub struct RepositoryQuery {
    // filters: フィルター条件リスト（AND 結合）
    pub filters: Vec<QueryFilter>,
    // order_by_column: ソートカラム名（None はデフォルト順序）
    pub order_by_column: Option<String>,
    // order_ascending: 昇順かどうか（true = ASC / false = DESC）
    pub order_ascending: bool,
    // page_size: ページサイズ（0 は制限なし）
    pub page_size: u64,
    // cursor: ページカーソル（None は先頭から）
    pub cursor: Option<String>,
}

// QueryableRepository<T> は高度なクエリ機能を持つ L1+ 拡張 trait。
// Repository<T> の基本 CRUD に加えてページネーション / フィルタリングを提供する。
// auth context 伝播と tracing は強制する。
#[async_trait]
pub trait QueryableRepository<T>: Repository<T> + Send + Sync
where
    // T はシリアライズ / デシリアライズ可能なドメイン型
    T: Serialize + DeserializeOwned + Send + Sync,
{
    // query はフィルター条件とページネーション設定でエンティティを取得する。
    // auth_ctx は tenant_id の境界を保証し、GUC SET LOCAL に使用する。
    // span_ctx は tracing コンテキストの伝播に使用する。
    async fn query(
        &self,
        query: RepositoryQuery,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
    ) -> Result<QueryPage<T>>;

    // delete は id と tenant_id を受け取り、エンティティを削除する。
    // 存在しない id を削除してもエラーにならない（idempotent）。
    async fn delete(
        &self,
        id: &str,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
    ) -> Result<()>;

    // count はフィルター条件に一致するエンティティ数を返す。
    async fn count(
        &self,
        filters: Vec<QueryFilter>,
        auth_ctx: &AuthContext,
    ) -> Result<u64>;
}
