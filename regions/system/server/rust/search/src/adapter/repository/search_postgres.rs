use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, QueryBuilder};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::entity::search_index::{
    PaginationResult, SearchDocument, SearchIndex, SearchQuery, SearchResult,
};
use crate::domain::repository::SearchRepository;

/// `SearchPostgresRepository` は `PostgreSQL` 全文検索を使った `SearchRepository` 実装。
pub struct SearchPostgresRepository {
    pool: Arc<PgPool>,
}

impl SearchPostgresRepository {
    #[must_use]
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
    tenant_id: String,
}

impl From<SearchIndexRow> for SearchIndex {
    fn from(r: SearchIndexRow) -> Self {
        SearchIndex {
            id: r.id,
            name: r.name,
            mapping: r.mapping,
            created_at: r.created_at,
            tenant_id: r.tenant_id,
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
            score: 1.0,
            indexed_at: r.created_at,
        }
    }
}

/// ファセット集計クエリの結果 Row 型。
#[derive(sqlx::FromRow)]
struct FacetRow {
    val: Option<String>,
    cnt: i64,
}

// SearchRepository 実装は多数のメソッドを持つため行数が多くなる
#[allow(clippy::too_many_lines)]
#[async_trait]
impl SearchRepository for SearchPostgresRepository {
    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してからインデックスを作成する。
    async fn create_index(&self, index: &SearchIndex, tenant_id: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO search.search_indices (id, name, mapping, tenant_id, created_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(index.id)
        .bind(&index.name)
        .bind(&index.mapping)
        .bind(tenant_id)
        .bind(index.created_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してからインデックスを名前で検索する。
    async fn find_index(&self, name: &str, tenant_id: &str) -> anyhow::Result<Option<SearchIndex>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row: Option<SearchIndexRow> = sqlx::query_as(
            "SELECT id, name, mapping, doc_count, created_at, updated_at, tenant_id \
             FROM search.search_indices WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してからドキュメントを登録する（UPSERT）。
    /// `search_vector` はトリガーが自動生成するため、INSERT では設定しない。
    async fn index_document(&self, doc: &SearchDocument, tenant_id: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        // インデックスの id を取得
        let index_row: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM search.search_indices WHERE name = $1")
                .bind(&doc.index_name)
                .fetch_optional(&mut *tx)
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
        .execute(&mut *tx)
        .await?;

        // doc_count を更新（実際のドキュメント数をカウント）
        sqlx::query(
            "UPDATE search.search_indices \
             SET doc_count = (SELECT COUNT(*) FROM search.search_documents WHERE index_id = $1) \
             WHERE id = $1",
        )
        .bind(index_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してから全文検索を実行する。
    /// - `plainto_tsquery` で任意のユーザー入力を安全に処理する（tsquery インジェクション防止）
    /// - query.filters の各キー・バリューを JSONB フィールドフィルタとして適用する
    /// - query.facets で指定されたフィールドの値ごとにドキュメント数を集計して返す
    async fn search(&self, query: &SearchQuery, tenant_id: &str) -> anyhow::Result<SearchResult> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let has_text_query = !query.query.trim().is_empty();

        // --- メインクエリ: QueryBuilder で動的 WHERE 節を構築 ---
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT d.id, d.document_id, d.content, d.created_at, i.name as index_name \
             FROM search.search_documents d \
             JOIN search.search_indices i ON i.id = d.index_id \
             WHERE i.name = ",
        );
        qb.push_bind(&query.index_name);

        // テキストクエリが存在する場合: plainto_tsquery でフルテキスト検索条件を追加
        if has_text_query {
            qb.push(" AND d.search_vector @@ plainto_tsquery('simple', ");
            qb.push_bind(&query.query);
            qb.push(")");
        }

        // フィルタ条件を追加: content->>'key' = 'value'
        for (key, value) in &query.filters {
            qb.push(" AND d.content->>");
            qb.push_bind(key);
            qb.push(" = ");
            qb.push_bind(value);
        }

        // 全文検索時は ts_rank 降順、全件取得時は作成日時降順
        if has_text_query {
            qb.push(" ORDER BY ts_rank(d.search_vector, plainto_tsquery('simple', ");
            qb.push_bind(&query.query);
            qb.push(")) DESC");
        } else {
            qb.push(" ORDER BY d.created_at DESC");
        }

        qb.push(" LIMIT ");
        qb.push_bind(i64::from(query.size));
        qb.push(" OFFSET ");
        qb.push_bind(i64::from(query.from));

        let rows: Vec<SearchDocumentRow> = qb.build_query_as().fetch_all(&mut *tx).await?;

        // --- カウントクエリ: メインクエリと同一の WHERE 条件 ---
        let mut count_qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT COUNT(*) FROM search.search_documents d \
             JOIN search.search_indices i ON i.id = d.index_id \
             WHERE i.name = ",
        );
        count_qb.push_bind(&query.index_name);

        if has_text_query {
            count_qb.push(" AND d.search_vector @@ plainto_tsquery('simple', ");
            count_qb.push_bind(&query.query);
            count_qb.push(")");
        }

        for (key, value) in &query.filters {
            count_qb.push(" AND d.content->>");
            count_qb.push_bind(key);
            count_qb.push(" = ");
            count_qb.push_bind(value);
        }

        let count: (i64,) = count_qb.build_query_as().fetch_one(&mut *tx).await?;

        // LOW-008: 安全な型変換（オーバーフロー防止）
        let total = u64::try_from(count.0.max(0)).unwrap_or(0);
        let hits: Vec<SearchDocument> = rows.into_iter().map(Into::into).collect();
        let page_size = query.size.max(1);
        let page = (query.from / page_size) + 1;
        let has_next = total > (u64::from(query.from) + u64::try_from(hits.len()).unwrap_or(u64::MAX));

        // --- ファセット集計: query.facets で指定されたフィールドごとに GROUP BY ---
        let mut facets: HashMap<String, HashMap<String, u64>> = HashMap::new();
        for facet_field in &query.facets {
            let mut facet_qb: QueryBuilder<sqlx::Postgres> =
                QueryBuilder::new("SELECT d.content->>");
            facet_qb.push_bind(facet_field);
            facet_qb.push(
                " AS val, COUNT(*) AS cnt \
                 FROM search.search_documents d \
                 JOIN search.search_indices i ON i.id = d.index_id \
                 WHERE i.name = ",
            );
            facet_qb.push_bind(&query.index_name);

            if has_text_query {
                facet_qb.push(" AND d.search_vector @@ plainto_tsquery('simple', ");
                facet_qb.push_bind(&query.query);
                facet_qb.push(")");
            }

            for (key, value) in &query.filters {
                facet_qb.push(" AND d.content->>");
                facet_qb.push_bind(key);
                facet_qb.push(" = ");
                facet_qb.push_bind(value);
            }

            // NULL 値を除外し、SELECT 節の 1 列目 (val) でグループ化
            facet_qb.push(" AND d.content->>");
            facet_qb.push_bind(facet_field);
            facet_qb.push(" IS NOT NULL GROUP BY 1");

            let facet_rows: Vec<FacetRow> = facet_qb.build_query_as().fetch_all(&mut *tx).await?;

            let field_counts: HashMap<String, u64> = facet_rows
                .into_iter()
                // LOW-008: 安全な型変換（オーバーフロー防止）
                .filter_map(|r| r.val.map(|v| (v, u64::try_from(r.cnt.max(0)).unwrap_or(0))))
                .collect();

            facets.insert(facet_field.clone(), field_counts);
        }

        tx.commit().await?;

        Ok(SearchResult {
            total,
            hits,
            facets,
            pagination: PaginationResult {
                total_count: total,
                page,
                page_size,
                has_next,
            },
        })
    }

    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してからドキュメントを削除する。
    async fn delete_document(
        &self,
        index_name: &str,
        doc_id: &str,
        tenant_id: &str,
    ) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        // インデックス id を取得
        let index_row: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM search.search_indices WHERE name = $1")
                .bind(index_name)
                .fetch_optional(&mut *tx)
                .await?;

        let index_id = if let Some(row) = index_row {
            row.0
        } else {
            tx.commit().await?;
            return Ok(false);
        };

        let result = sqlx::query(
            "DELETE FROM search.search_documents WHERE index_id = $1 AND document_id = $2",
        )
        .bind(index_id)
        .bind(doc_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            // doc_count を更新
            sqlx::query(
                "UPDATE search.search_indices \
                 SET doc_count = (SELECT COUNT(*) FROM search.search_documents WHERE index_id = $1) \
                 WHERE id = $1",
            )
            .bind(index_id)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            Ok(true)
        } else {
            tx.commit().await?;
            Ok(false)
        }
    }

    /// CRIT-005 対応: トランザクション内で `set_config` を呼び出してテナント分離してから全インデックスを取得する。
    async fn list_indices(&self, tenant_id: &str) -> anyhow::Result<Vec<SearchIndex>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows: Vec<SearchIndexRow> = sqlx::query_as(
            "SELECT id, name, mapping, doc_count, created_at, updated_at, tenant_id \
             FROM search.search_indices ORDER BY created_at DESC",
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
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
            tenant_id: "tenant-a".to_string(),
        };
        let index: SearchIndex = row.into();
        assert_eq!(index.name, "products");
        assert_eq!(index.mapping, serde_json::json!({"fields": ["name"]}));
        assert_eq!(index.tenant_id, "tenant-a");
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
    fn test_has_text_query_detection() {
        // 空白のみのクエリはテキスト検索なしとみなす
        assert!(!("".trim().is_empty() == false));
        assert!("  ".trim().is_empty());
        assert!(!"hello".trim().is_empty());
    }

    #[test]
    fn test_facet_row_to_map() {
        // FacetRow の val が None の行はファセット集計から除外されることを確認
        let rows = vec![
            FacetRow {
                val: Some("electronics".to_string()),
                cnt: 5,
            },
            FacetRow { val: None, cnt: 3 },
            FacetRow {
                val: Some("books".to_string()),
                cnt: 2,
            },
        ];
        let map: HashMap<String, u64> = rows
            .into_iter()
            // LOW-008: 安全な型変換（オーバーフロー防止）
            .filter_map(|r| r.val.map(|v| (v, u64::try_from(r.cnt.max(0)).unwrap_or(0))))
            .collect();
        assert_eq!(map.get("electronics"), Some(&5u64));
        assert_eq!(map.get("books"), Some(&2u64));
        // None の行は含まれない
        assert_eq!(map.len(), 2);
    }
}
