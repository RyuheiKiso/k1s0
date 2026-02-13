use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn test_new_backend_rust_non_interactive() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("k1s0")
        .unwrap()
        .args([
            "new", "backend",
            "--template", "rust",
            "--name", "my-rust-svc",
            "--path", tmp.path().to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-rust-svc"));

    let project_dir = tmp.path().join("my-rust-svc");
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src/main.rs").exists());
    assert!(project_dir.join("src/lib.rs").exists());
    assert!(project_dir.join("tests/health_check.rs").exists());
    assert!(project_dir.join("README.md").exists());
    assert!(project_dir.join(".github/workflows/ci.yml").exists());
    assert!(project_dir.join("Dockerfile").exists());
    assert!(project_dir.join(".dockerignore").exists());
    assert!(project_dir.join("docker-compose.yml").exists());
    assert!(project_dir.join("k8s/namespace.yml").exists());
    assert!(project_dir.join("k8s/deployment.yml").exists());
    assert!(project_dir.join("k8s/service.yml").exists());
    assert!(project_dir.join("k8s/ingress.yml").exists());
    assert!(project_dir.join("k8s/configmap.yml").exists());
}

#[test]
fn test_new_backend_rust_with_postgresql() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("k1s0")
        .unwrap()
        .args([
            "new", "backend",
            "--template", "rust",
            "--name", "my-db-svc",
            "--db", "postgresql",
            "--path", tmp.path().to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .success();

    let project_dir = tmp.path().join("my-db-svc");
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("Dockerfile").exists());
    assert!(project_dir.join(".dockerignore").exists());
    assert!(project_dir.join("docker-compose.yml").exists());
    assert!(project_dir.join("migrations/001_init.sql").exists());
    assert!(project_dir.join("k8s/namespace.yml").exists());
    assert!(project_dir.join("k8s/postgres-secret.yml").exists());
    assert!(project_dir.join("k8s/postgres-pvc.yml").exists());
    assert!(project_dir.join("k8s/postgres-statefulset.yml").exists());
    assert!(project_dir.join("k8s/postgres-service.yml").exists());
}

#[test]
fn test_new_backend_go_non_interactive() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("k1s0")
        .unwrap()
        .args([
            "new", "backend",
            "--template", "go",
            "--name", "my-go-svc",
            "--path", tmp.path().to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .success();

    let project_dir = tmp.path().join("my-go-svc");
    assert!(project_dir.join("go.mod").exists());
    assert!(project_dir.join("main.go").exists());
    assert!(project_dir.join("main_test.go").exists());
    assert!(project_dir.join("README.md").exists());
    assert!(project_dir.join(".github/workflows/ci.yml").exists());
    assert!(project_dir.join("Dockerfile").exists());
    assert!(project_dir.join(".dockerignore").exists());
    assert!(project_dir.join("docker-compose.yml").exists());
}

#[test]
fn test_new_backend_missing_name_yes_flag_errors() {
    Command::cargo_bin("k1s0")
        .unwrap()
        .args(["new", "backend", "--template", "rust", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("name is required"));
}

#[test]
fn test_new_backend_incompatible_template_errors() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("k1s0")
        .unwrap()
        .args([
            "new", "backend",
            "--template", "react",
            "--name", "bad-svc",
            "--path", tmp.path().to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not compatible"));
}
