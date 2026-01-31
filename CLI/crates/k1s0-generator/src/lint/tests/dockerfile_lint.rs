use super::super::*;
use super::{create_backend_rust_structure, create_test_manifest};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_rule_id_k060() {
    assert_eq!(RuleId::DockerfileBaseImageUnpinned.as_str(), "K060");
}

#[test]
fn test_latest_tag_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    fs::write(path.join("Dockerfile"), "FROM rust:latest\nRUN cargo build\n").unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::DockerfileBaseImageUnpinned),
        "Expected K060 violation for :latest tag",
    );
}

#[test]
fn test_no_tag_detected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    fs::write(path.join("Dockerfile"), "FROM ubuntu\nRUN apt-get update\n").unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::DockerfileBaseImageUnpinned),
        "Expected K060 violation for missing tag",
    );
}

#[test]
fn test_pinned_version_ok() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    fs::write(path.join("Dockerfile"), "FROM rust:1.85.0\nRUN cargo build\n").unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::DockerfileBaseImageUnpinned),
        "Pinned version should not trigger K060",
    );
}

#[test]
fn test_sha256_digest_ok() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    fs::write(
        path.join("Dockerfile"),
        "FROM rust@sha256:abc123def456\nRUN cargo build\n",
    )
    .unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::DockerfileBaseImageUnpinned),
        "sha256 digest should not trigger K060",
    );
}

#[test]
fn test_multistage_build() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    let dockerfile = "FROM rust:1.85.0 AS builder\nRUN cargo build\nFROM debian:latest\nCOPY --from=builder /app /app\n";
    fs::write(path.join("Dockerfile"), dockerfile).unwrap();

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    let violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.rule == RuleId::DockerfileBaseImageUnpinned)
        .collect();

    assert_eq!(violations.len(), 1, "Only debian:latest should trigger K060");
}

#[test]
fn test_no_dockerfile_no_violation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    create_test_manifest(path, "backend-rust");
    create_backend_rust_structure(path);

    // Dockerfile を削除（create_backend_rust_structure が作らない場合もあるが念のため）
    let _ = fs::remove_file(path.join("Dockerfile"));

    let linter = Linter::default_linter();
    let result = linter.lint(path);

    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.rule == RuleId::DockerfileBaseImageUnpinned),
    );
}
