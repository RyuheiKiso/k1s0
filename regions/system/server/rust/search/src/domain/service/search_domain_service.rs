use std::collections::{HashMap, HashSet};

use crate::domain::entity::search_index::SearchQuery;

pub struct SearchDomainService;

impl SearchDomainService {
    #[must_use] 
    pub fn normalize_query(raw: &str) -> String {
        raw.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[must_use] 
    pub fn normalize_filters(raw: &HashMap<String, String>) -> HashMap<String, String> {
        raw.iter()
            .filter_map(|(k, v)| {
                let key = k.trim();
                let value = v.trim();
                if key.is_empty() || value.is_empty() {
                    None
                } else {
                    Some((key.to_string(), value.to_string()))
                }
            })
            .collect()
    }

    #[must_use] 
    pub fn normalize_facets(raw: &[String]) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut facets = Vec::new();

        for facet in raw {
            let normalized = facet.trim();
            if normalized.is_empty() {
                continue;
            }
            if seen.insert(normalized.to_string()) {
                facets.push(normalized.to_string());
            }
        }

        facets
    }

    #[must_use] 
    pub fn build_query(
        index_name: String,
        query: &str,
        from: u32,
        size: u32,
        filters: &HashMap<String, String>,
        facets: &[String],
    ) -> SearchQuery {
        SearchQuery {
            index_name,
            query: Self::normalize_query(query),
            from,
            size,
            filters: Self::normalize_filters(filters),
            facets: Self::normalize_facets(facets),
        }
    }
}
