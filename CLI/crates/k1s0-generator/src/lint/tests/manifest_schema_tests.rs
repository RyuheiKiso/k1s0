//! Manifest schema validation tests
//!
//! Tests that manifest.schema.json is valid and contains expected definitions.

use std::path::PathBuf;

/// Get the path to manifest.schema.json.
fn schema_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // CLI/
        .join("schemas")
        .join("manifest.schema.json")
}

#[test]
fn test_schema_is_valid_json() {
    let content = std::fs::read_to_string(schema_path()).expect("Should read manifest.schema.json");
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("manifest.schema.json should be valid JSON");
}

#[test]
fn test_schema_features_contains_all_flags() {
    let content = std::fs::read_to_string(schema_path()).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&content).unwrap();

    let features = &schema["properties"]["service"]["properties"]["features"]["properties"];
    let features_obj = features
        .as_object()
        .expect("features properties should be an object");

    let expected = ["with_grpc", "with_rest", "with_db", "with_cache", "with_docker"];
    for key in &expected {
        assert!(
            features_obj.contains_key(*key),
            "Schema features should define {}",
            key
        );
    }
}

#[test]
fn test_schema_managed_paths_example_contains_dockerfile() {
    let content = std::fs::read_to_string(schema_path()).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&content).unwrap();

    let managed_paths = &schema["properties"]["managed_paths"];

    // Check examples array for Dockerfile
    let examples = managed_paths["examples"]
        .as_array()
        .expect("managed_paths should have examples");

    let has_dockerfile = examples.iter().any(|example| {
        if let Some(arr) = example.as_array() {
            arr.iter()
                .any(|item| item.as_str() == Some("Dockerfile"))
        } else {
            false
        }
    });

    assert!(
        has_dockerfile,
        "managed_paths examples should contain Dockerfile"
    );
}
