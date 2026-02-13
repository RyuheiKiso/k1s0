use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn test_new_frontend_react_non_interactive() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("k1s0")
        .unwrap()
        .args([
            "new", "frontend",
            "--template", "react",
            "--name", "my-react-app",
            "--path", tmp.path().to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-react-app"));

    let project_dir = tmp.path().join("my-react-app");
    assert!(project_dir.join("package.json").exists());
    assert!(project_dir.join("index.html").exists());
    assert!(project_dir.join("vite.config.ts").exists());
    assert!(project_dir.join("tsconfig.json").exists());
    assert!(project_dir.join("src/main.tsx").exists());
    assert!(project_dir.join("src/App.tsx").exists());
    assert!(project_dir.join("src/App.test.tsx").exists());
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
fn test_new_frontend_flutter_non_interactive() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("k1s0")
        .unwrap()
        .args([
            "new", "frontend",
            "--template", "flutter",
            "--name", "my-flutter-app",
            "--path", tmp.path().to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .success();

    let project_dir = tmp.path().join("my-flutter-app");
    assert!(project_dir.join("pubspec.yaml").exists());
    assert!(project_dir.join("lib/main.dart").exists());
    assert!(project_dir.join("test/widget_test.dart").exists());
    assert!(project_dir.join("README.md").exists());
    assert!(project_dir.join(".github/workflows/ci.yml").exists());
    assert!(project_dir.join("Dockerfile").exists());
    assert!(project_dir.join(".dockerignore").exists());
    assert!(project_dir.join("docker-compose.yml").exists());
}

#[test]
fn test_new_frontend_missing_name_yes_flag_errors() {
    Command::cargo_bin("k1s0")
        .unwrap()
        .args(["new", "frontend", "--template", "react", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("name is required"));
}

#[test]
fn test_new_frontend_invalid_template_errors() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("k1s0")
        .unwrap()
        .args([
            "new", "frontend",
            "--template", "rust",
            "--name", "bad-app",
            "--path", tmp.path().to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not compatible"));
}

#[test]
fn test_new_frontend_invalid_name_errors() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("k1s0")
        .unwrap()
        .args([
            "new", "frontend",
            "--template", "react",
            "--name", "bad app!",
            "--path", tmp.path().to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid characters"));
}
