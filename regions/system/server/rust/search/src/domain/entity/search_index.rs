use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub id: Uuid,
    pub name: String,
    pub mapping: serde_json::Value,
    pub created_at: DateTime<Utc>,
    /// CRIT-005 対応: RLS でテナント分離するためのテナント ID。
    pub tenant_id: String,
}

impl SearchIndex {
    /// `SearchIndex` を生成する。
    /// `tenant_id` を必須引数として受け取り、全インデックスが正しいテナントに帰属するようにする。
    /// CRIT-002 対応: ハードコードされた "system" テナントを廃止し、呼び出し元が必ず `tenant_id` を渡す設計に変更する。
    #[must_use]
    pub fn new(name: String, mapping: serde_json::Value, tenant_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            mapping,
            created_at: Utc::now(),
            tenant_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub index_name: String,
    pub content: serde_json::Value,
    pub score: f32,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub index_name: String,
    pub query: String,
    pub from: u32,
    pub size: u32,
    pub filters: HashMap<String, String>,
    pub facets: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SearchResult {
    pub total: u64,
    pub hits: Vec<SearchDocument>,
    pub facets: HashMap<String, HashMap<String, u64>>,
    pub pagination: PaginationResult,
}

#[derive(Debug, Clone)]
pub struct PaginationResult {
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}
