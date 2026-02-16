/// ライブラリテンプレートのレンダリング統合テスト。
///
/// 実際の CLI/templates/library/{go,rust,typescript,dart}/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書と一致することを検証する。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

// =========================================================================
// ヘルパー関数
// =========================================================================

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_library(lang: &str) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("shared-utils", "system", lang, "library")
        .build();

    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

    let names: Vec<String> = generated
        .iter()
        .map(|p| {
            p.strip_prefix(&output_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    (tmp, names)
}

fn read_output(tmp: &TempDir, path: &str) -> String {
    fs::read_to_string(tmp.path().join("output").join(path)).unwrap()
}

// =========================================================================
// Go ライブラリ
// =========================================================================

#[test]
fn test_library_go_file_list() {
    let (_, names) = render_library("go");

    assert!(names.iter().any(|n| n == "go.mod"), "go.mod missing");
    assert!(names.iter().any(|n| n == "shared-utils.go"), "{{name}}.go -> shared-utils.go missing");
    assert!(names.iter().any(|n| n == "internal/internal.go"), "internal/internal.go missing");
    assert!(names.iter().any(|n| n == "shared-utils_test.go"), "{{name}}_test.go -> shared-utils_test.go missing");
    assert!(names.iter().any(|n| n == "tests/integration_test.go"), "tests/integration_test.go missing");
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_library_go_no_tera_syntax() {
    let (tmp, names) = render_library("go");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {}", name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

#[test]
fn test_library_go_service_name_substitution() {
    let (tmp, _) = render_library("go");

    let go_mod = read_output(&tmp, "go.mod");
    // go_module が正しく導出されている
    assert!(go_mod.contains("module github.com/org/k1s0/regions/system/library/go/shared-utils"));
}

// =========================================================================
// Rust ライブラリ
// =========================================================================

#[test]
fn test_library_rust_file_list() {
    let (_, names) = render_library("rust");

    assert!(names.iter().any(|n| n == "Cargo.toml"), "Cargo.toml missing");
    assert!(names.iter().any(|n| n == "src/lib.rs"), "src/lib.rs missing");
    assert!(names.iter().any(|n| n == "src/shared_utils.rs"), "{{module}}.rs -> shared_utils.rs missing");
    assert!(names.iter().any(|n| n == "tests/integration_test.rs"), "tests/integration_test.rs missing");
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_library_rust_no_tera_syntax() {
    let (tmp, names) = render_library("rust");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {}", name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

#[test]
fn test_library_rust_service_name_substitution() {
    let (tmp, _) = render_library("rust");

    let cargo = read_output(&tmp, "Cargo.toml");
    // rust_crate が正しく導出されている
    assert!(cargo.contains("name = \"shared-utils\""));
}

// =========================================================================
// TypeScript ライブラリ
// =========================================================================

#[test]
fn test_library_typescript_file_list() {
    let (_, names) = render_library("typescript");

    assert!(names.iter().any(|n| n == "package.json"), "package.json missing");
    assert!(names.iter().any(|n| n == "tsconfig.json"), "tsconfig.json missing");
    assert!(names.iter().any(|n| n == "src/index.ts"), "src/index.ts missing");
    assert!(names.iter().any(|n| n == "tests/index.test.ts"), "tests/index.test.ts missing");
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_library_typescript_no_tera_syntax() {
    let (tmp, names) = render_library("typescript");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {}", name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

#[test]
fn test_library_typescript_service_name_substitution() {
    let (tmp, _) = render_library("typescript");

    let package = read_output(&tmp, "package.json");
    assert!(package.contains("\"name\": \"shared-utils\""));
}

// =========================================================================
// Dart ライブラリ
// =========================================================================

#[test]
fn test_library_dart_file_list() {
    let (_, names) = render_library("dart");

    assert!(names.iter().any(|n| n == "pubspec.yaml"), "pubspec.yaml missing");
    assert!(names.iter().any(|n| n == "analysis_options.yaml"), "analysis_options.yaml missing");
    assert!(names.iter().any(|n| n == "lib/shared-utils.dart"), "{{name}}.dart -> shared-utils.dart missing");
    assert!(names.iter().any(|n| n == "lib/src/shared_utils.dart"), "lib/src/{{module}}.dart -> shared_utils.dart missing");
    assert!(names.iter().any(|n| n == "test/shared_utils_test.dart"), "test/{{module}}_test.dart -> shared_utils_test.dart missing");
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_library_dart_no_tera_syntax() {
    let (tmp, names) = render_library("dart");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {}", name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

#[test]
fn test_library_dart_service_name_substitution() {
    let (tmp, _) = render_library("dart");

    let pubspec = read_output(&tmp, "pubspec.yaml");
    // service_name の snake_case が pubspec name に使用される
    assert!(pubspec.contains("shared_utils"));
}
