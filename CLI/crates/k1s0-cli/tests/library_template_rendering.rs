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

    let ctx = TemplateContextBuilder::new("shared-utils", "system", lang, "library").build();

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
    assert!(
        names.iter().any(|n| n == "shared-utils.go"),
        "shared-utils.go missing (from name.go.tera)"
    );
    assert!(
        names.iter().any(|n| n == "internal/internal.go"),
        "internal/internal.go missing"
    );
    assert!(
        names.iter().any(|n| n == "shared-utils_test.go"),
        "shared-utils_test.go missing (from name_test.go.tera)"
    );
    assert!(
        names.iter().any(|n| n == "tests/integration_test.go"),
        "tests/integration_test.go missing"
    );
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_library_go_no_tera_syntax() {
    let (tmp, names) = render_library("go");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {name}");
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {name}");
        assert!(!content.contains("{#"), "Tera comment {{# found in {name}");
    }
}

#[test]
fn test_library_go_service_name_substitution() {
    let (tmp, _) = render_library("go");

    let go_mod = read_output(&tmp, "go.mod");
    // go_module が正しく導出されている
    assert!(go_mod.contains("module github.com/org/k1s0/regions/system/library/go/shared-utils"));
}

#[test]
fn test_library_go_public_package_content() {
    let (tmp, _) = render_library("go");
    let content = read_output(&tmp, "shared-utils.go");

    assert!(content.contains("package shared_utils") || content.contains("package "));
    assert!(
        content.contains("Config") && content.contains("Client") && content.contains("func New")
    );
}

#[test]
fn test_library_go_internal_content() {
    let (tmp, _) = render_library("go");
    let content = read_output(&tmp, "internal/internal.go");

    assert!(content.contains("package internal"));
}

#[test]
fn test_library_go_unit_test_content() {
    let (tmp, _) = render_library("go");
    let content = read_output(&tmp, "shared-utils_test.go");

    assert!(content.contains("package ") || content.contains("func Test"));
}

#[test]
fn test_library_go_readme_content() {
    let (tmp, _) = render_library("go");
    let content = read_output(&tmp, "README.md");

    assert!(content.contains("shared-utils"));
}

// =========================================================================
// Rust ライブラリ
// =========================================================================

#[test]
fn test_library_rust_file_list() {
    let (_, names) = render_library("rust");

    assert!(
        names.iter().any(|n| n == "Cargo.toml"),
        "Cargo.toml missing"
    );
    assert!(
        names.iter().any(|n| n == "src/lib.rs"),
        "src/lib.rs missing"
    );
    assert!(
        names.iter().any(|n| n == "src/shared_utils.rs"),
        "shared_utils.rs missing (from module.rs.tera)"
    );
    assert!(
        names.iter().any(|n| n == "tests/integration_test.rs"),
        "tests/integration_test.rs missing"
    );
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_library_rust_no_tera_syntax() {
    let (tmp, names) = render_library("rust");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {name}");
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {name}");
        assert!(!content.contains("{#"), "Tera comment {{# found in {name}");
    }
}

#[test]
fn test_library_rust_service_name_substitution() {
    let (tmp, _) = render_library("rust");

    let cargo = read_output(&tmp, "Cargo.toml");
    // rust_crate が正しく導出されている
    assert!(cargo.contains("name = \"shared-utils\""));
}

#[test]
fn test_library_rust_cargo_toml_content() {
    let (tmp, _) = render_library("rust");
    let content = read_output(&tmp, "Cargo.toml");

    assert!(content.contains("edition = \"2021\""));
    assert!(content.contains("serde"));
    assert!(content.contains("thiserror"));
    assert!(content.contains("[dev-dependencies]"));
    assert!(content.contains("mockall"));
}

#[test]
fn test_library_rust_lib_rs_content() {
    let (tmp, _) = render_library("rust");
    let content = read_output(&tmp, "src/lib.rs");

    assert!(content.contains("pub mod"));
}

#[test]
fn test_library_rust_module_content() {
    let (tmp, _) = render_library("rust");
    let content = read_output(&tmp, "src/shared_utils.rs");

    assert!(content.contains("#[cfg(test)]"));
    assert!(content.contains("mod tests"));
}

#[test]
fn test_library_rust_readme_content() {
    let (tmp, _) = render_library("rust");
    let content = read_output(&tmp, "README.md");

    assert!(content.contains("shared-utils"));
}

// =========================================================================
// TypeScript ライブラリ
// =========================================================================

#[test]
fn test_library_typescript_file_list() {
    let (_, names) = render_library("typescript");

    assert!(
        names.iter().any(|n| n == "package.json"),
        "package.json missing"
    );
    assert!(
        names.iter().any(|n| n == "tsconfig.json"),
        "tsconfig.json missing"
    );
    assert!(
        names.iter().any(|n| n == "src/index.ts"),
        "src/index.ts missing"
    );
    assert!(
        names.iter().any(|n| n == "tests/index.test.ts"),
        "tests/index.test.ts missing"
    );
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_library_typescript_no_tera_syntax() {
    let (tmp, names) = render_library("typescript");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {name}");
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {name}");
        assert!(!content.contains("{#"), "Tera comment {{# found in {name}");
    }
}

#[test]
fn test_library_typescript_service_name_substitution() {
    let (tmp, _) = render_library("typescript");

    let package = read_output(&tmp, "package.json");
    assert!(package.contains("\"name\": \"shared-utils\""));
}

#[test]
fn test_library_typescript_package_json_content() {
    let (tmp, _) = render_library("typescript");
    let content = read_output(&tmp, "package.json");

    assert!(content.contains("\"main\""));
    assert!(content.contains("dist/index.js"));
    assert!(content.contains("\"types\""));
    assert!(content.contains("dist/index.d.ts"));
    assert!(content.contains("\"build\""));
    assert!(content.contains("\"test\""));
    assert!(content.contains("typescript"));
    assert!(content.contains("vitest"));
}

#[test]
fn test_library_typescript_tsconfig_content() {
    let (tmp, _) = render_library("typescript");
    let content = read_output(&tmp, "tsconfig.json");

    assert!(content.contains("\"strict\": true"));
    assert!(content.contains("\"declaration\": true"));
    assert!(content.contains("\"outDir\": \"dist\""));
}

#[test]
fn test_library_typescript_index_ts_content() {
    let (tmp, _) = render_library("typescript");
    let content = read_output(&tmp, "src/index.ts");

    assert!(content.contains("shared-utils") || content.contains("export"));
}

#[test]
fn test_library_typescript_readme_content() {
    let (tmp, _) = render_library("typescript");
    let content = read_output(&tmp, "README.md");

    assert!(content.contains("shared-utils"));
}

// =========================================================================
// Dart ライブラリ
// =========================================================================

#[test]
fn test_library_dart_file_list() {
    let (_, names) = render_library("dart");

    assert!(
        names.iter().any(|n| n == "pubspec.yaml"),
        "pubspec.yaml missing"
    );
    assert!(
        names.iter().any(|n| n == "analysis_options.yaml"),
        "analysis_options.yaml missing"
    );
    assert!(
        names.iter().any(|n| n == "lib/shared-utils.dart"),
        "shared-utils.dart missing (from name.dart.tera)"
    );
    assert!(
        names.iter().any(|n| n == "lib/src/shared_utils.dart"),
        "shared_utils.dart missing (from module.dart.tera)"
    );
    assert!(
        names.iter().any(|n| n == "test/shared_utils_test.dart"),
        "shared_utils_test.dart missing (from module_test.dart.tera)"
    );
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_library_dart_no_tera_syntax() {
    let (tmp, names) = render_library("dart");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {name}");
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {name}");
        assert!(!content.contains("{#"), "Tera comment {{# found in {name}");
    }
}

#[test]
fn test_library_dart_service_name_substitution() {
    let (tmp, _) = render_library("dart");

    let pubspec = read_output(&tmp, "pubspec.yaml");
    // service_name の snake_case が pubspec name に使用される
    assert!(pubspec.contains("shared_utils"));
}

#[test]
fn test_library_dart_pubspec_content() {
    let (tmp, _) = render_library("dart");
    let content = read_output(&tmp, "pubspec.yaml");

    assert!(content.contains(">=3.0.0 <4.0.0") || content.contains("sdk:"));
    assert!(content.contains("mocktail") || content.contains("dev_dependencies"));
}

#[test]
fn test_library_dart_entry_point_content() {
    let (tmp, _) = render_library("dart");
    let content = read_output(&tmp, "lib/shared-utils.dart");

    assert!(content.contains("library"));
}

#[test]
fn test_library_dart_src_module_content() {
    let (tmp, _) = render_library("dart");
    let content = read_output(&tmp, "lib/src/shared_utils.dart");

    assert!(content.contains("shared_utils"));
}

#[test]
fn test_library_dart_analysis_options_content() {
    let (tmp, _) = render_library("dart");
    let content = read_output(&tmp, "analysis_options.yaml");

    assert!(!content.is_empty());
}

#[test]
fn test_library_dart_readme_content() {
    let (tmp, _) = render_library("dart");
    let content = read_output(&tmp, "README.md");

    assert!(content.contains("shared-utils") || content.contains("shared_utils"));
}

// =========================================================================
// エラー型・バリデーション検証
// =========================================================================

#[test]
fn test_library_go_has_app_error() {
    let (tmp, _) = render_library("go");
    let content = read_output(&tmp, "shared-utils.go");
    assert!(
        content.contains("AppError"),
        "Go library should contain AppError struct"
    );
    assert!(
        content.contains("func (e *AppError) Error() string"),
        "Go AppError should implement error interface"
    );
}

#[test]
fn test_library_go_has_validate() {
    let (tmp, _) = render_library("go");
    let content = read_output(&tmp, "shared-utils.go");
    assert!(
        content.contains("func (c Config) Validate() error"),
        "Go Config should have Validate() method"
    );
}

#[test]
fn test_library_go_test_has_error_cases() {
    let (tmp, _) = render_library("go");
    let content = read_output(&tmp, "shared-utils_test.go");
    assert!(
        content.contains("TestValidate_Error"),
        "Go test should contain error validation test"
    );
}

#[test]
fn test_library_rust_has_lib_error() {
    let (tmp, _) = render_library("rust");
    let content = read_output(&tmp, "src/shared_utils.rs");
    assert!(
        content.contains("LibError"),
        "Rust library should contain LibError enum"
    );
    assert!(
        content.contains("thiserror::Error"),
        "Rust LibError should derive thiserror::Error"
    );
}

#[test]
fn test_library_rust_has_validate() {
    let (tmp, _) = render_library("rust");
    let content = read_output(&tmp, "src/shared_utils.rs");
    assert!(
        content.contains("pub fn validate(&self) -> Result<(), LibError>"),
        "Rust Config should have validate() method"
    );
}

#[test]
fn test_library_rust_test_has_error_cases() {
    let (tmp, _) = render_library("rust");
    let content = read_output(&tmp, "src/shared_utils.rs");
    assert!(
        content.contains("test_validate_error"),
        "Rust test should contain error validation test"
    );
}

#[test]
fn test_library_typescript_has_app_error() {
    let (tmp, _) = render_library("typescript");
    let content = read_output(&tmp, "src/index.ts");
    assert!(
        content.contains("class AppError extends Error"),
        "TypeScript should contain AppError class"
    );
}

#[test]
fn test_library_typescript_has_validate() {
    let (tmp, _) = render_library("typescript");
    let content = read_output(&tmp, "src/index.ts");
    assert!(
        content.contains("validate"),
        "TypeScript should have validate function"
    );
}

#[test]
fn test_library_typescript_test_has_error_cases() {
    let (tmp, _) = render_library("typescript");
    let content = read_output(&tmp, "tests/index.test.ts");
    assert!(
        content.contains("error") || content.contains("AppError"),
        "TypeScript test should contain error test cases"
    );
}

#[test]
fn test_library_dart_has_app_exception() {
    let (tmp, _) = render_library("dart");
    let content = read_output(&tmp, "lib/src/shared_utils.dart");
    assert!(
        content.contains("class AppException implements Exception"),
        "Dart should contain AppException class"
    );
}

#[test]
fn test_library_dart_has_validate() {
    let (tmp, _) = render_library("dart");
    let content = read_output(&tmp, "lib/src/shared_utils.dart");
    assert!(
        content.contains("validate"),
        "Dart Config should have validate() method"
    );
}

#[test]
fn test_library_dart_test_has_error_cases() {
    let (tmp, _) = render_library("dart");
    let content = read_output(&tmp, "test/shared_utils_test.dart");
    assert!(
        content.contains("AppException") || content.contains("error"),
        "Dart test should contain error test cases"
    );
}

// =========================================================================
// Python ライブラリ
// =========================================================================

#[test]
fn test_library_python_file_list() {
    let (_, names) = render_library("python");
    assert!(
        names.iter().any(|n| n.contains("pyproject.toml")),
        "pyproject.toml missing"
    );
    assert!(
        names.iter().any(|n| n.contains("README.md")),
        "README.md missing"
    );
    assert!(
        names.iter().any(|n| n.contains("__init__.py")),
        "__init__.py missing"
    );
    assert!(
        names.iter().any(|n| n.contains("exceptions.py")),
        "exceptions.py missing"
    );
    assert!(
        names.iter().any(|n| n.contains("test_")),
        "test file missing"
    );
}

#[test]
fn test_library_python_no_tera_syntax() {
    let (tmp, names) = render_library("python");
    for name in &names {
        let content = fs::read_to_string(tmp.path().join("output").join(name)).unwrap();
        assert!(
            !content.contains("{{"),
            "Unresolved Tera syntax in {name}: {{{{}}}}"
        );
        assert!(!content.contains("}}"), "Unresolved Tera syntax in {name}");
    }
}

#[test]
fn test_library_python_service_name_substitution() {
    let (tmp, _) = render_library("python");
    let content = read_output(&tmp, "pyproject.toml");
    assert!(
        content.contains("shared-utils") || content.contains("shared_utils"),
        "Service name not substituted in pyproject.toml"
    );
}

#[test]
fn test_library_python_pyproject_content() {
    let (tmp, _) = render_library("python");
    let content = read_output(&tmp, "pyproject.toml");
    assert!(
        content.contains("[project]"),
        "pyproject.toml should have [project] section"
    );
    assert!(
        content.contains("python"),
        "pyproject.toml should reference python version"
    );
}

#[test]
fn test_library_python_has_error_class() {
    let (tmp, _) = render_library("python");
    let content = read_output(&tmp, "src/k1s0_shared_utils/exceptions.py");
    assert!(
        content.contains("Error"),
        "exceptions.py should contain Error class"
    );
    assert!(
        content.contains("code"),
        "exceptions.py should have error code"
    );
}

#[test]
fn test_library_python_has_validate() {
    let (tmp, _) = render_library("python");
    let content = read_output(&tmp, "src/k1s0_shared_utils/shared_utils.py");
    assert!(
        content.contains("validate"),
        "Python library should have validate method"
    );
}

#[test]
fn test_library_python_test_has_error_cases() {
    let (tmp, _) = render_library("python");
    let content = read_output(&tmp, "tests/test_shared_utils.py");
    assert!(
        content.contains("Error") || content.contains("error"),
        "Python test should contain error test cases"
    );
}


