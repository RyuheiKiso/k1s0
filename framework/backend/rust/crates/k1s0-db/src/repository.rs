//! リポジトリパターン
//!
//! データアクセスを抽象化するリポジトリパターンの実装。
//!
//! # 機能
//!
//! - **Repository トレイト**: CRUD操作の抽象化
//! - **Pagination**: ページネーション付きクエリ結果
//! - **SoftDelete**: 論理削除サポート
//!
//! # 使用例
//!
//! ```rust,ignore
//! use k1s0_db::repository::{Repository, PagedResult, Pagination};
//!
//! #[derive(Debug, Clone)]
//! struct User {
//!     id: String,
//!     name: String,
//!     email: String,
//! }
//!
//! struct UserRepository {
//!     pool: PgPool,
//! }
//!
//! #[async_trait::async_trait]
//! impl Repository<User, String> for UserRepository {
//!     async fn find_by_id(&self, id: &String) -> Result<Option<User>, DbError> {
//!         // 実装
//!     }
//!     // ...
//! }
//! ```

use crate::error::{DbError, DbResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// ページネーション設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pagination {
    /// ページ番号（1から開始）
    pub page: u64,
    /// ページサイズ
    pub page_size: u64,
}

impl Pagination {
    /// 新しいページネーションを作成
    pub fn new(page: u64, page_size: u64) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.min(1000).max(1),
        }
    }

    /// オフセットを計算
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.page_size
    }

    /// リミットを取得
    pub fn limit(&self) -> u64 {
        self.page_size
    }

    /// デフォルトのページサイズで最初のページを作成
    pub fn first_page() -> Self {
        Self::new(1, 20)
    }

    /// 次のページを取得
    pub fn next_page(&self) -> Self {
        Self::new(self.page + 1, self.page_size)
    }

    /// 前のページを取得（最初のページより前には戻らない）
    pub fn prev_page(&self) -> Self {
        Self::new(self.page.saturating_sub(1).max(1), self.page_size)
    }
}

/// ページネーション付き結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    /// データ
    pub data: Vec<T>,
    /// 総件数
    pub total: u64,
    /// 現在のページ番号
    pub page: u64,
    /// ページサイズ
    pub page_size: u64,
    /// 総ページ数
    pub total_pages: u64,
}

impl<T> PagedResult<T> {
    /// 新しいページング結果を作成
    pub fn new(data: Vec<T>, total: u64, pagination: &Pagination) -> Self {
        let total_pages = if pagination.page_size > 0 {
            (total + pagination.page_size - 1) / pagination.page_size
        } else {
            0
        };

        Self {
            data,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
            total_pages,
        }
    }

    /// 空の結果を作成
    pub fn empty(pagination: &Pagination) -> Self {
        Self::new(Vec::new(), 0, pagination)
    }

    /// 次のページがあるかどうか
    pub fn has_next_page(&self) -> bool {
        self.page < self.total_pages
    }

    /// 前のページがあるかどうか
    pub fn has_prev_page(&self) -> bool {
        self.page > 1
    }

    /// データが空かどうか
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// データの件数
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// データをマッピング
    pub fn map<U, F>(self, f: F) -> PagedResult<U>
    where
        F: FnMut(T) -> U,
    {
        PagedResult {
            data: self.data.into_iter().map(f).collect(),
            total: self.total,
            page: self.page,
            page_size: self.page_size,
            total_pages: self.total_pages,
        }
    }
}

/// ソート方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    /// 昇順
    Asc,
    /// 降順
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Asc
    }
}

impl std::fmt::Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asc => write!(f, "ASC"),
            Self::Desc => write!(f, "DESC"),
        }
    }
}

/// ソート設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortBy {
    /// ソートするカラム
    pub column: String,
    /// ソート方向
    pub direction: SortDirection,
}

impl SortBy {
    /// 新しいソート設定を作成
    pub fn new(column: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }

    /// 昇順ソートを作成
    pub fn asc(column: impl Into<String>) -> Self {
        Self::new(column, SortDirection::Asc)
    }

    /// 降順ソートを作成
    pub fn desc(column: impl Into<String>) -> Self {
        Self::new(column, SortDirection::Desc)
    }
}

/// リポジトリトレイト
///
/// エンティティのCRUD操作を抽象化する。
///
/// # 型パラメータ
///
/// * `T` - エンティティ型
/// * `ID` - IDの型（通常は `String`, `i64`, `Uuid` など）
#[async_trait]
pub trait Repository<T, ID>: Send + Sync
where
    T: Send + Sync,
    ID: Send + Sync,
{
    /// IDでエンティティを取得
    async fn find_by_id(&self, id: &ID) -> DbResult<Option<T>>;

    /// 全エンティティを取得
    async fn find_all(&self) -> DbResult<Vec<T>>;

    /// ページネーション付きで全エンティティを取得
    async fn find_all_paged(&self, pagination: &Pagination) -> DbResult<PagedResult<T>>;

    /// エンティティを保存（作成または更新）
    async fn save(&self, entity: &T) -> DbResult<T>;

    /// 複数のエンティティを一括保存
    async fn save_all(&self, entities: &[T]) -> DbResult<Vec<T>> {
        let mut results = Vec::with_capacity(entities.len());
        for entity in entities {
            results.push(self.save(entity).await?);
        }
        Ok(results)
    }

    /// IDでエンティティを削除
    async fn delete(&self, id: &ID) -> DbResult<bool>;

    /// エンティティが存在するかチェック
    async fn exists(&self, id: &ID) -> DbResult<bool> {
        Ok(self.find_by_id(id).await?.is_some())
    }

    /// エンティティの総数を取得
    async fn count(&self) -> DbResult<u64>;
}

/// 論理削除サポート
#[async_trait]
pub trait SoftDeleteRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
{
    /// 論理削除されたエンティティを含めて取得
    async fn find_by_id_including_deleted(&self, id: &ID) -> DbResult<Option<T>>;

    /// 論理削除されたエンティティのみを取得
    async fn find_all_deleted(&self) -> DbResult<Vec<T>>;

    /// エンティティを論理削除
    async fn soft_delete(&self, id: &ID) -> DbResult<bool>;

    /// 論理削除されたエンティティを復元
    async fn restore(&self, id: &ID) -> DbResult<bool>;

    /// 論理削除されたエンティティを物理削除
    async fn hard_delete(&self, id: &ID) -> DbResult<bool>;
}

/// フィルタリング可能なリポジトリ
#[async_trait]
pub trait FilterableRepository<T, ID, F>: Repository<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
    F: Send + Sync,
{
    /// フィルタ条件でエンティティを検索
    async fn find_by_filter(&self, filter: &F) -> DbResult<Vec<T>>;

    /// フィルタ条件でページネーション付き検索
    async fn find_by_filter_paged(
        &self,
        filter: &F,
        pagination: &Pagination,
    ) -> DbResult<PagedResult<T>>;

    /// フィルタ条件に一致するエンティティの数を取得
    async fn count_by_filter(&self, filter: &F) -> DbResult<u64>;
}

/// ソート可能なリポジトリ
#[async_trait]
pub trait SortableRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
{
    /// ソート付きで全エンティティを取得
    async fn find_all_sorted(&self, sort_by: &[SortBy]) -> DbResult<Vec<T>>;

    /// ソートとページネーション付きで全エンティティを取得
    async fn find_all_sorted_paged(
        &self,
        sort_by: &[SortBy],
        pagination: &Pagination,
    ) -> DbResult<PagedResult<T>>;
}

/// 一括操作サポート
#[async_trait]
pub trait BulkRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync,
{
    /// 複数のIDでエンティティを取得
    async fn find_by_ids(&self, ids: &[ID]) -> DbResult<Vec<T>>;

    /// 複数のIDでエンティティを削除
    async fn delete_by_ids(&self, ids: &[ID]) -> DbResult<u64>;

    /// 全エンティティを削除
    async fn delete_all(&self) -> DbResult<u64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(1, 20);
        assert_eq!(pagination.offset(), 0);
        assert_eq!(pagination.limit(), 20);

        let pagination = Pagination::new(2, 20);
        assert_eq!(pagination.offset(), 20);

        let pagination = Pagination::new(3, 10);
        assert_eq!(pagination.offset(), 20);
    }

    #[test]
    fn test_pagination_bounds() {
        // Page 0 should become page 1
        let pagination = Pagination::new(0, 20);
        assert_eq!(pagination.page, 1);

        // Page size should be bounded
        let pagination = Pagination::new(1, 2000);
        assert_eq!(pagination.page_size, 1000);

        let pagination = Pagination::new(1, 0);
        assert_eq!(pagination.page_size, 1);
    }

    #[test]
    fn test_pagination_navigation() {
        let pagination = Pagination::new(2, 20);
        let next = pagination.next_page();
        assert_eq!(next.page, 3);

        let prev = pagination.prev_page();
        assert_eq!(prev.page, 1);

        // First page should not go below 1
        let first = Pagination::new(1, 20);
        let prev = first.prev_page();
        assert_eq!(prev.page, 1);
    }

    #[test]
    fn test_paged_result() {
        let pagination = Pagination::new(1, 10);
        let result: PagedResult<i32> = PagedResult::new(vec![1, 2, 3], 25, &pagination);

        assert_eq!(result.total, 25);
        assert_eq!(result.total_pages, 3);
        assert!(result.has_next_page());
        assert!(!result.has_prev_page());
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_paged_result_map() {
        let pagination = Pagination::new(1, 10);
        let result: PagedResult<i32> = PagedResult::new(vec![1, 2, 3], 3, &pagination);

        let mapped = result.map(|x| x * 2);
        assert_eq!(mapped.data, vec![2, 4, 6]);
        assert_eq!(mapped.total, 3);
    }

    #[test]
    fn test_sort_by() {
        let sort = SortBy::asc("name");
        assert_eq!(sort.column, "name");
        assert_eq!(sort.direction, SortDirection::Asc);

        let sort = SortBy::desc("created_at");
        assert_eq!(sort.direction, SortDirection::Desc);
    }
}
