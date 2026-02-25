use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};
use crate::domain::repository::SearchRepository;

/// SearchPostgresRepository は PostgreSQL 全文検索を使った SearchRepository 実装。
pub struct SearchPostgresRepository {
    pool: Arc<PgPool>,
}

impl SearchPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

// --- Row types for sqlx ---

#[derive(sqlx::FromRow)]
struct SearchIndexRow {
    id: Uuid,
    name: String,
    mapping: serde_json::Value,
    #[allow(dead_code)]
    doc_count: i64,
    created_at: DateTime<Utc>,
    #[allow(dead_code)]
    updated_at: DateTime<Utc>,
}

impl From<SearchIndexRow> for SearchIndex {
    fn from(r: SearchIndexRow) -> Self {
        SearchIndex {
            id: r.id,
            name: r.name,
            mapping: r.mapping,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SearchDocumentRow {
    #[allow(dead_code)]
    id: Uuid,
    document_id: String,
    content: serde_json::Value,
    created_at: DateTime<Utc>,
    index_name: String,
}

impl From<SearchDocumentRow> for SearchDocument {
    fn from(r: SearchDocumentRow) -> Self {
        SearchDocument {
            id: r.document_id,
            index_name: r.index_name,
            content: r.content,
            indexed_at: r.created_at,
        }
    }
}

#[async_trait]
impl SearchRepository for SearchPostgresRepository {
    /// インデックスを作成する。
    async fn create_index(&self, index: &SearchIndex) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO search.search_indices (id, name, mapping, created_at) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(index.id)
        .bind(&index.name)
        .bind(&index.mapping)
        .bind(index.created_at)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    /// インデックスを名前で検索する。
    async fn find_index(&self, name: &str) -> anyhow::Result<Option<SearchIndex>> {
        let row: Option<SearchIndexRow> = sqlx::query_as(
            "SELECT id, name, mapping, doc_count, created_at, updated_at \
             FROM search.search_indices WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await?;
        Ok(row.map(Into::into))
    }

    /// ドキュメントをインデックスに登録する（UPSERT）。
    /// search_vector はトリガーが自動生成するため、INSERT では設定しない。
    async fn index_document(&self, doc: &SearchDocument) -> anyhow::Result<()> {
        // まずインデックスの id を取得
        let index_row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM search.search_indices WHERE name = $1",
        )
        .bind(&doc.index_name)
        .fetch_optional(self.pool.as_ref())
        .await?;

        let index_id = index_row
            .ok_or_else(|| anyhow::anyhow!("index not found: {}", doc.index_name))?
            .0;

        // UPSERT: ON CONFLICT で content を更新
        sqlx::query(
            "INSERT INTO search.search_documents (index_id, document_id, content) \
             VALUES ($1, $2, $3) \
             ON CONFLICT ON CONSTRAINT uq_search_documents_index_doc \
             DO UPDATE SET content = EXCLUDED.content",
        )
        .bind(index_id)
        .bind(&doc.id)
        .bind(&doc.content)
        .execute(self.pool.as_ref())
        .await?;

        // doc_count を更新（実際のドキュメント数をカウント）
        sqlx::query(
            "UPDATE search.search_indices \
             SET doc_count = (SELECT COUNT(*) FROM search.search_documents WHERE index_id = $1) \
             WHERE id = $1",
        )
        .bind(index_id)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// PostgreSQL 全文検索を使ってドキュメントを検索する。
    /// tsquery の prefix マッチ (:*) と ts_rank による関連度順ソートをサポート。
    async fn search(&self, query: &SearchQuery) -> anyhow::Result<SearchResult> {
        // ユーザークエリを tsquery に変換（各単語を prefix マッチで AND 結合）
        let ts_query = query
            .query
            .split_whitespace()
            .map(|w| format!("{}:*", w))
            .collect::<Vec<_>>()
            .join(" & ");

        // 空クエリの場合は全件返す
        if ts_query.is_empty() {
            let rows: Vec<SearchDocumentRow> = sqlx::query_as(
                "SELECT d.id, d.document_id, d.content, d.created_at, i.name as index_name \
                 FROM search.search_documents d \
                 JOIN search.search_indices i ON i.id = d.index_id \
                 WHERE i.name = $1 \
                 ORDER BY d.created_at DESC \
                 LIMIT $2 OFFSET $3",
            )
            .bind(&query.index_name)
            .bind(query.size as i64)
            .bind(query.from as i64)
            .fetch_all(self.pool.as_ref())
            .await?;

            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM search.search_documents d \
                 JOIN search.search_indices i ON i.id = d.index_id \
                 WHERE i.name = $1",
            )
            .bind(&query.index_name)
            .fetch_one(self.pool.as_ref())
            .await?;

            return Ok(SearchResult {
                total: count.0 as u64,
                hits: rows.into_iter().map(Into::into).collect(),
            });
        }

        // 全文検索: tsquery マッチでランキング
        let rows: Vec<SearchDocumentRow> = sqlx::query_as(
            "SELECT d.id, d.document_id, d.content, d.created_at, i.name as index_name \
             FROM search.search_documents d \
             JOIN search.search_indices i ON i.id = d.index_id \
             WHERE i.name = $1 AND d.search_vector @@ to_tsquery('simple', $2) \
             ORDER BY ts_rank(d.search_vector, to_tsquery('simple', $2)) DESC \
             LIMIT $3 OFFSET $4",
        )
        .bind(&query.index_name)
        .bind(&ts_query)
        .bind(query.size as i64)
        .bind(query.from as i64)
        .fetch_all(self.pool.as_ref())
        .await?;

        // トータルカウント
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM search.search_documents d \
             JOIN search.search_indices i ON i.id = d.index_id \
             WHERE i.name = $1 AND d.search_vector @@ to_tsquery('simple', $2)",
        )
        .bind(&query.index_name)
        .bind(&ts_query)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(SearchResult {
            total: count.0 as u64,
            hits: rows.into_iter().map(Into::into).collect(),
        })
    }

    /// ドキュメントを削除する。
    async fn delete_document(&self, index_name: &str, doc_id: &str) -> anyhow::Result<bool> {
        // インデックス id を取得
        let index_row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM search.search_indices WHERE name = $1",
        )
        .bind(index_name)
        .fetch_optional(self.pool.as_ref())
        .await?;

        let index_id = match index_row {
            Some(row) => row.0,
            None => return Ok(false),
        };

        let result = sqlx::query(
            "DELETE FROM search.search_documents WHERE index_id = $1 AND document_id = $2",
        )
        .bind(index_id)
        .bind(doc_id)
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() > 0 {
            // doc_count を更新
            sqlx::query(
                "UPDATE search.search_indices \
                 SET doc_count = (SELECT COUNT(*) FROM search.search_documents WHERE index_id = $1) \
                 WHERE id = $1",
            )
            .bind(index_id)
            .execute(self.pool.as_ref())
            .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// すべてのインデックスを取得する。
    async fn list_indices(&self) -> anyhow::Result<Vec<SearchIndex>> {
        let rows: Vec<SearchIndexRow> = sqlx::query_as(
            "SELECT id, name, mapping, doc_count, created_at, updated_at \
             FROM search.search_indices ORDER BY created_at DESC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_index_row_conversion() {
        let row = SearchIndexRow {
            id: Uuid::new_v4(),
            name: "products".to_string(),
            mapping: serde_json::json!({"fields": ["name"]}),
            doc_count: 42,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let index: SearchIndex = row.into();
        assert_eq!(index.name, "products");
        assert_eq!(index.mapping, serde_json::json!({"fields": ["name"]}));
    }

    #[test]
    fn test_search_document_row_conversion() {
        let row = SearchDocumentRow {
            id: Uuid::new_v4(),
            document_id: "doc-1".to_string(),
            content: serde_json::json!({"title": "Test"}),
            created_at: Utc::now(),
            index_name: "products".to_string(),
        };
        let doc: SearchDocument = row.into();
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.index_name, "products");
        assert_eq!(doc.content, serde_json::json!({"title": "Test"}));
    }

    #[test]
    fn test_tsquery_construction() {
        // tsquery 変換ロジックの検証
        let query_str = "Widget blue";
        let ts_query = query_str
            .split_whitespace()
            .map(|w| format!("{}:*", w))
            .collect::<Vec<_>>()
            .join(" & ");
        assert_eq!(ts_query, "Widget:* & blue:*");
    }

    #[test]
    fn test_tsquery_single_word() {
        let query_str = "Widget";
        let ts_query = query_str
            .split_whitespace()
            .map(|w| format!("{}:*", w))
            .collect::<Vec<_>>()
            .join(" & ");
        assert_eq!(ts_query, "Widget:*");
    }

    #[test]
    fn test_tsquery_empty() {
        let query_str = "";
        let ts_query = query_str
            .split_whitespace()
            .map(|w| format!("{}:*", w))
            .collect::<Vec<_>>()
            .join(" & ");
        assert_eq!(ts_query, "");
    }
}
