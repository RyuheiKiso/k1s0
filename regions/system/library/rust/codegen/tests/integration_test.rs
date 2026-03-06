use k1s0_codegen::{generate, ApiStyle, DatabaseType, ScaffoldConfig, Tier};
use std::path::Path;

fn minimal_config() -> ScaffoldConfig {
    ScaffoldConfig {
        name: "my-service".into(),
        tier: Tier::System,
        api_style: ApiStyle::Rest,
        database: DatabaseType::None,
        description: "A test service".into(),
        proto_path: None,
        generate_client: false,
    }
}

fn full_config() -> ScaffoldConfig {
    ScaffoldConfig {
        name: "order-manager".into(),
        tier: Tier::Business,
        api_style: ApiStyle::Both,
        database: DatabaseType::Postgres,
        description: "Order management service".into(),
        proto_path: None,
        generate_client: false,
    }
}

#[test]
fn minimal_scaffold_creates_expected_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("my-service");

    let result = generate(&minimal_config(), &output).unwrap();

    assert!(result.skipped.is_empty());
    assert!(!result.created.is_empty());

    // Always-generated files
    let expected = vec![
        "Cargo.toml",
        "src/main.rs",
        "src/lib.rs",
        "src/error.rs",
        "config/config.yaml",
        "README.md",
        "src/adapter/handler/mod.rs",
        "src/infrastructure/config.rs",
        "src/adapter/mod.rs",
        "src/domain/mod.rs",
        "src/domain/entity/mod.rs",
        "src/domain/repository/mod.rs",
        "src/domain/service/mod.rs",
        "src/usecase/mod.rs",
        "src/infrastructure/mod.rs",
    ];
    for f in &expected {
        assert!(
            output.join(f).exists(),
            "expected file missing: {}",
            f
        );
    }

    // gRPC files should NOT exist
    assert!(!output.join("build.rs").exists());
    assert!(!output.join("src/proto/.gitkeep").exists());

    // DB files should NOT exist
    assert!(!output.join("migrations/001_initial.up.sql").exists());
}

#[test]
fn full_scaffold_creates_grpc_and_db_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("order-manager");

    let result = generate(&full_config(), &output).unwrap();

    // gRPC files
    assert!(output.join("build.rs").exists());
    assert!(output.join("src/proto/.gitkeep").exists());
    assert!(output
        .join("api/proto/k1s0/business/order_manager/v1/order_manager.proto")
        .exists());

    // DB files
    assert!(output.join("migrations/001_initial.up.sql").exists());
    assert!(output.join("migrations/001_initial.down.sql").exists());

    assert!(result.skipped.is_empty());
}

#[test]
fn idempotent_second_run_skips_existing() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("my-service");

    let first = generate(&minimal_config(), &output).unwrap();
    let created_count = first.created.len();
    assert!(created_count > 0);

    let second = generate(&minimal_config(), &output).unwrap();
    assert!(second.created.is_empty());
    assert_eq!(second.skipped.len(), created_count);
}

#[test]
fn cargo_toml_is_valid_toml() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("my-service");
    generate(&minimal_config(), &output).unwrap();

    let content = std::fs::read_to_string(output.join("Cargo.toml")).unwrap();
    let parsed: toml::Value = content.parse().expect("Cargo.toml should be valid TOML");
    let pkg = parsed.get("package").expect("should have [package]");
    assert_eq!(
        pkg.get("name").unwrap().as_str().unwrap(),
        "k1s0-my-service-server"
    );
}

#[test]
fn config_yaml_is_valid_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("my-service");
    generate(&minimal_config(), &output).unwrap();

    let content = std::fs::read_to_string(output.join("config/config.yaml")).unwrap();
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("config.yaml should be valid YAML");
    let app = parsed.get("app").expect("should have app section");
    assert_eq!(
        app.get("name").unwrap().as_str().unwrap(),
        "k1s0-my-service-server"
    );
}

#[test]
fn validation_rejects_invalid_names() {
    let configs = vec![
        ("", "empty name"),
        ("My-Server", "uppercase"),
        ("-server", "leading hyphen"),
        ("server-", "trailing hyphen"),
        ("my--server", "consecutive hyphens"),
    ];

    for (name, desc) in configs {
        let config = ScaffoldConfig {
            name: name.into(),
            ..minimal_config()
        };
        assert!(
            generate(&config, Path::new("/tmp/dummy")).is_err(),
            "should reject: {}",
            desc
        );
    }
}

#[test]
fn output_path_calculation() {
    use k1s0_codegen::build_output_path;
    let p = build_output_path(Path::new("/repo"), Tier::System, "auth");
    assert!(p.ends_with("regions/system/server/rust/auth"));
}
