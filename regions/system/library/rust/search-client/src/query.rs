use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
    pub value_to: Option<serde_json::Value>,
}

impl Filter {
    pub fn eq(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Self {
            field: field.into(),
            operator: "eq".to_string(),
            value: value.into(),
            value_to: None,
        }
    }

    pub fn lt(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Self {
            field: field.into(),
            operator: "lt".to_string(),
            value: value.into(),
            value_to: None,
        }
    }

    pub fn gt(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        Self {
            field: field.into(),
            operator: "gt".to_string(),
            value: value.into(),
            value_to: None,
        }
    }

    pub fn range(
        field: impl Into<String>,
        from: impl Into<serde_json::Value>,
        to: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            field: field.into(),
            operator: "range".to_string(),
            value: from.into(),
            value_to: Some(to.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetBucket {
    pub value: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub filters: Vec<Filter>,
    pub facets: Vec<String>,
    pub page: u32,
    pub size: u32,
}

impl SearchQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            filters: Vec::new(),
            facets: Vec::new(),
            page: 0,
            size: 20,
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn facet(mut self, facet: impl Into<String>) -> Self {
        self.facets.push(facet.into());
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    pub fn size(mut self, size: u32) -> Self {
        self.size = size;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub hits: Vec<T>,
    pub total: u64,
    pub facets: std::collections::HashMap<String, Vec<FacetBucket>>,
    pub took_ms: u64,
}
