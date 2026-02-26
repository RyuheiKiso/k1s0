use async_trait::async_trait;
use chrono::Utc;
use opensearch::auth::Credentials;
use opensearch::cert::CertificateValidation;
use opensearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use opensearch::http::Url;
use opensearch::indices::IndicesCreateParts;
use opensearch::cat::CatIndicesParts;
use opensearch::{DeleteParts, IndexParts, OpenSearch, SearchParts};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};
use crate::domain::repository::SearchRepository;

/// SearchOpenSearchRepository は OpenSearch を使った SearchRepository 実装。
pub struct SearchOpenSearchRepository {
    client: OpenSearch,
    prefix: String,
}

impl SearchOpenSearchRepository {
    pub fn new(url: &str, username: &str, password: &str, prefix: &str) -> anyhow::Result<Self> {
        let url = Url::parse(url)?;
        let conn_pool = SingleNodeConnectionPool::new(url);
        let mut builder = TransportBuilder::new(conn_pool)
            .cert_validation(CertificateValidation::None);

        if !username.is_empty() && !password.is_empty() {
            builder = builder.auth(Credentials::Basic(
                username.to_string(),
                password.to_string(),
            ));
        }

        let transport = builder.build()?;
        let client = OpenSearch::new(transport);

        Ok(Self {
            client,
            prefix: prefix.to_string(),
        })
    }

    /// プレフィックス付きインデックス名を返す。
    fn index_name(&self, name: &str) -> String {
        format!("{}{}", self.prefix, name)
    }
}

#[async_trait]
impl SearchRepository for SearchOpenSearchRepository {
    async fn create_index(&self, index: &SearchIndex) -> anyhow::Result<()> {
        let idx_name = self.index_name(&index.name);
        let response = self
            .client
            .indices()
            .create(IndicesCreateParts::Index(&idx_name))
            .body(json!({ "mappings": index.mapping }))
            .send()
            .await?;

        if !response.status_code().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("failed to create index {}: {}", idx_name, body);
        }
        Ok(())
    }

    async fn find_index(&self, name: &str) -> anyhow::Result<Option<SearchIndex>> {
        let idx_name = self.index_name(name);
        let response = self
            .client
            .indices()
            .exists(opensearch::indices::IndicesExistsParts::Index(&[&idx_name]))
            .send()
            .await?;

        if response.status_code().is_success() {
            Ok(Some(SearchIndex {
                id: Uuid::new_v4(),
                name: name.to_string(),
                mapping: json!({}),
                created_at: Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn index_document(&self, doc: &SearchDocument) -> anyhow::Result<()> {
        let idx_name = self.index_name(&doc.index_name);
        let response = self
            .client
            .index(IndexParts::IndexId(&idx_name, &doc.id))
            .body(doc.content.clone())
            .send()
            .await?;

        if !response.status_code().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "failed to index document {} in {}: {}",
                doc.id,
                idx_name,
                body
            );
        }
        Ok(())
    }

    async fn search(&self, query: &SearchQuery) -> anyhow::Result<SearchResult> {
        let idx_name = self.index_name(&query.index_name);

        let body = if query.query.is_empty() {
            json!({ "query": { "match_all": {} } })
        } else {
            json!({
                "query": {
                    "multi_match": {
                        "query": query.query,
                        "fields": ["*"]
                    }
                }
            })
        };

        let response = self
            .client
            .search(SearchParts::Index(&[&idx_name]))
            .from(query.from as i64)
            .size(query.size as i64)
            .body(body)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("search failed on {}: {}", idx_name, body);
        }

        let response_body: Value = response.json().await?;

        let total = response_body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(0);

        let hits = response_body["hits"]["hits"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|hit| SearchDocument {
                        id: hit["_id"].as_str().unwrap_or("").to_string(),
                        index_name: query.index_name.clone(),
                        content: hit["_source"].clone(),
                        indexed_at: Utc::now(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(SearchResult { total, hits })
    }

    async fn delete_document(&self, index_name: &str, doc_id: &str) -> anyhow::Result<bool> {
        let idx_name = self.index_name(index_name);
        let response = self
            .client
            .delete(DeleteParts::IndexId(&idx_name, doc_id))
            .send()
            .await?;

        if response.status_code().is_success() {
            let body: Value = response.json().await?;
            Ok(body["result"].as_str() == Some("deleted"))
        } else {
            Ok(false)
        }
    }

    async fn list_indices(&self) -> anyhow::Result<Vec<SearchIndex>> {
        let pattern = format!("{}*", self.prefix);
        let response = self
            .client
            .cat()
            .indices(CatIndicesParts::Index(&[&pattern]))
            .format("json")
            .send()
            .await?;

        if !response.status_code().is_success() {
            return Ok(vec![]);
        }

        let body: Value = response.json().await?;
        let indices = body
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| {
                        let full_name = item["index"].as_str().unwrap_or("");
                        let name = full_name
                            .strip_prefix(&self.prefix)
                            .unwrap_or(full_name)
                            .to_string();
                        SearchIndex {
                            id: Uuid::new_v4(),
                            name,
                            mapping: json!({}),
                            created_at: Utc::now(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_name_with_prefix() {
        // OpenSearch クライアント構築をスキップして index_name ロジックだけテスト
        let repo = SearchOpenSearchRepository {
            client: OpenSearch::default(),
            prefix: "k1s0-".to_string(),
        };
        assert_eq!(repo.index_name("products"), "k1s0-products");
        assert_eq!(repo.index_name("logs"), "k1s0-logs");
    }

    #[test]
    fn test_index_name_empty_prefix() {
        let repo = SearchOpenSearchRepository {
            client: OpenSearch::default(),
            prefix: "".to_string(),
        };
        assert_eq!(repo.index_name("products"), "products");
    }

    #[test]
    fn test_index_name_custom_prefix() {
        let repo = SearchOpenSearchRepository {
            client: OpenSearch::default(),
            prefix: "test-env-".to_string(),
        };
        assert_eq!(repo.index_name("users"), "test-env-users");
    }
}
