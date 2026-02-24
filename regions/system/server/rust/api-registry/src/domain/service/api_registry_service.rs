//! API schema domain service.

use crate::domain::entity::api_registration::{
    BreakingChange, ChangeDetail, CompatibilityResult, DiffEntry, DiffModifiedEntry, SchemaDiff,
    SchemaType,
};

pub struct ApiRegistryDomainService;

impl ApiRegistryDomainService {
    pub fn new() -> Self { Self }
    pub fn check_compatibility(
        &self, schema_type: &SchemaType, old_content: &str, new_content: &str,
    ) -> CompatibilityResult {
        match schema_type {
            SchemaType::OpenApi => self.check_openapi_compatibility(old_content, new_content),
            SchemaType::Protobuf => self.check_protobuf_compatibility(old_content, new_content),
        }
    }

    pub fn compute_diff(
        &self, schema_type: &SchemaType, old_content: &str, new_content: &str,
    ) -> SchemaDiff {
        match schema_type {
            SchemaType::OpenApi => self.compute_openapi_diff(old_content, new_content),
            SchemaType::Protobuf => self.compute_protobuf_diff(old_content, new_content),
        }
    }

    fn check_openapi_compatibility(&self, old_content: &str, new_content: &str) -> CompatibilityResult {
        let mut breaking_changes = Vec::new();
        let mut non_breaking_changes = Vec::new();
        let old: serde_json::Value = serde_yaml::from_str(old_content).unwrap_or(serde_json::Value::Null);
        let new_val: serde_json::Value = serde_yaml::from_str(new_content).unwrap_or(serde_json::Value::Null);
        if old == serde_json::Value::Null || new_val == serde_json::Value::Null {
            return CompatibilityResult { compatible: true, breaking_changes, non_breaking_changes };
        }
        if let (Some(old_paths), Some(new_paths)) = (
            old.get("paths").and_then(|p| p.as_object()),
            new_val.get("paths").and_then(|p| p.as_object()),
        ) {
            for (path, _) in old_paths {
                if \!new_paths.contains_key(path.as_str()) {
                    breaking_changes.push(BreakingChange::new(
                        "path_removed".to_string(), path.clone(),
                        format\!("API path {} was removed", path),
                    ));
                } else {
                    let old_m = old_paths.get(path.as_str()).and_then(|p| p.as_object());
                    let new_m = new_paths.get(path.as_str()).and_then(|p| p.as_object());
                    if let (Some(om), Some(nm)) = (old_m, new_m) {
                        for method in ["get", "post", "put", "delete", "patch"] {
                            if om.contains_key(method) && \!nm.contains_key(method) {
                                breaking_changes.push(BreakingChange::new(
                                    "method_removed".to_string(),
                                    format\!("{} {}", method.to_uppercase(), path),
                                    format\!("HTTP method {} was removed from {}", method.to_uppercase(), path),
                                ));
                            }
                        }
                    }
                }
            }
            for (path, _) in new_paths {
                if \!old_paths.contains_key(path.as_str()) {
                    non_breaking_changes.push(ChangeDetail {
                        change_type: "path_added".to_string(), path: path.clone(),
                        description: format\!("New API path {} was added", path),
                    });
                }
            }
        }
        CompatibilityResult { compatible: breaking_changes.is_empty(), breaking_changes, non_breaking_changes }
    }

    fn check_protobuf_compatibility(&self, old_content: &str, new_content: &str) -> CompatibilityResult {
        let mut breaking_changes = Vec::new();
        let non_breaking_changes = Vec::new();
        let old_fields = extract_proto_fields(old_content);
        let new_fields = extract_proto_fields(new_content);
        for (field_num, field_name) in &old_fields {
            if \!new_fields.iter().any(|(n, _)| n == field_num) {
                breaking_changes.push(BreakingChange::new(
                    "field_removed".to_string(),
                    format\!("field {}", field_name),
                    format\!("Proto field {} (number {}) was removed", field_name, field_num),
                ));
            }
        }
        CompatibilityResult { compatible: breaking_changes.is_empty(), breaking_changes, non_breaking_changes }
    }

    fn compute_openapi_diff(&self, old_content: &str, new_content: &str) -> SchemaDiff {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let removed = Vec::new();
        let old: serde_json::Value = serde_yaml::from_str(old_content).unwrap_or(serde_json::Value::Null);
        let new_val: serde_json::Value = serde_yaml::from_str(new_content).unwrap_or(serde_json::Value::Null);
        if let (Some(old_paths), Some(new_paths)) = (
            old.get("paths").and_then(|p| p.as_object()),
            new_val.get("paths").and_then(|p| p.as_object()),
        ) {
            for (path, _) in new_paths {
                if \!old_paths.contains_key(path.as_str()) {
                    added.push(DiffEntry { path: path.clone(), entry_type: "path".to_string(), description: format\!("New path {}", path) });
                }
            }
            for (path, _) in old_paths {
                if new_paths.contains_key(path.as_str()) {
                    let os = old_paths.get(path.as_str()).and_then(|p| p.get("get")).and_then(|g| g.get("summary")).and_then(|s| s.as_str());
                    let ns = new_paths.get(path.as_str()).and_then(|p| p.get("get")).and_then(|g| g.get("summary")).and_then(|s| s.as_str());
                    if let (Some(o), Some(n)) = (os, ns) {
                        if o \!= n { modified.push(DiffModifiedEntry { path: format\!("{} GET summary", path), before: o.to_string(), after: n.to_string() }); }
                    }
                }
            }
        }
        SchemaDiff { added, modified, removed }
    }

    fn compute_protobuf_diff(&self, old_content: &str, new_content: &str) -> SchemaDiff {
        let old_fields = extract_proto_fields(old_content);
        let new_fields = extract_proto_fields(new_content);
        let mut added = Vec::new();
        let removed = Vec::new();
        for (field_num, field_name) in &new_fields {
            if \!old_fields.iter().any(|(n, _)| n == field_num) {
                added.push(DiffEntry { path: format\!("field {}", field_name), entry_type: "field".to_string(), description: format\!("New proto field {} (number {})", field_name, field_num) });
            }
        }
        SchemaDiff { added, modified: Vec::new(), removed }
    }
}

impl Default for ApiRegistryDomainService {
    fn default() -> Self { Self::new() }
}

fn extract_proto_fields(content: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let re_field = regex::Regex::new(r"\s+\w+\s+(\w+)\s*=\s*(\d+);")
        .unwrap_or_else(|_| regex::Regex::new(r"x").unwrap());
    for cap in re_field.captures_iter(content) {
        let field_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let field_num = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
        fields.push((field_num, field_name));
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_compatibility_no_changes() {
        let service = ApiRegistryDomainService::new();
        let content = "openapi: 3.0.3\npaths:\n  /api/v1/test:\n    get:\n      summary: Test\n";
        let result = service.check_compatibility(&SchemaType::OpenApi, content, content);
        assert!(result.compatible);
        assert!(result.breaking_changes.is_empty());
    }

    #[test]
    fn test_openapi_compatibility_path_removed() {
        let service = ApiRegistryDomainService::new();
        let old = "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users\n  /api/v1/orders:\n    get:\n      summary: Orders\n";
        let new_v = "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users\n";
        let result = service.check_compatibility(&SchemaType::OpenApi, old, new_v);
        assert!(!result.compatible);
        assert_eq!(result.breaking_changes.len(), 1);
        assert_eq!(result.breaking_changes[0].change_type, "path_removed");
    }

    #[test]
    fn test_protobuf_compatibility_field_removed() {
        let service = ApiRegistryDomainService::new();
let old = "syntax = \"proto3\";
message Test {
  string id = 1;
  string name = 2;
}
";
let new_v = "syntax = \"proto3\";
message Test {
  string id = 1;
}
";
        let result = service.check_compatibility(&SchemaType::Protobuf, old, new_v);
        assert!(!result.compatible);
        assert_eq!(result.breaking_changes.len(), 1);
    }
}
