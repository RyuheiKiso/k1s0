/// クライアントテンプレートのレンダリング統合テスト。
///
/// 実際の CLI/templates/client/{react,flutter}/ テンプレートファイルを使用し、
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

fn render_client(framework: &str) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("test-app", "service", framework, "client")
        .framework(framework)
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
// React クライアント
// =========================================================================

#[test]
fn test_client_react_file_list() {
    let (_, names) = render_client("react");

    // 必須ファイルの存在確認
    assert!(names.iter().any(|n| n == "package.json"), "package.json missing");
    assert!(names.iter().any(|n| n == "tsconfig.json"), "tsconfig.json missing");
    assert!(names.iter().any(|n| n == "vite.config.ts"), "vite.config.ts missing");
    assert!(names.iter().any(|n| n == "vitest.config.ts"), "vitest.config.ts missing");
    assert!(names.iter().any(|n| n == "eslint.config.mjs"), "eslint.config.mjs missing");
    assert!(names.iter().any(|n| n == ".prettierrc"), ".prettierrc missing");
    assert!(names.iter().any(|n| n == "src/app/App.tsx"), "src/app/App.tsx missing");
    assert!(names.iter().any(|n| n == "src/lib/api-client.ts"), "src/lib/api-client.ts missing");
    assert!(names.iter().any(|n| n == "src/lib/query-client.ts"), "src/lib/query-client.ts missing");
    assert!(names.iter().any(|n| n == "Dockerfile"), "Dockerfile missing");
    assert!(names.iter().any(|n| n == "nginx.conf"), "nginx.conf missing");
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_client_react_package_json_content() {
    let (tmp, _) = render_client("react");
    let content = read_output(&tmp, "package.json");

    assert!(content.contains("\"name\": \"test-app\""));
    assert!(content.contains("\"react\": \"^19.0.0\""));
    assert!(content.contains("\"react-dom\": \"^19.0.0\""));
    assert!(content.contains("\"@tanstack/react-query\""));
    assert!(content.contains("\"@tanstack/react-router\""));
    assert!(content.contains("\"vite\""));
    assert!(content.contains("\"vitest\""));
    assert!(content.contains("\"typescript\""));
    assert!(content.contains("\"tailwindcss\""));
}

#[test]
fn test_client_react_tsconfig_content() {
    let (tmp, _) = render_client("react");
    let content = read_output(&tmp, "tsconfig.json");

    assert!(content.contains("\"target\": \"ES2022\""));
    assert!(content.contains("\"jsx\": \"react-jsx\""));
    assert!(content.contains("\"strict\": true"));
    assert!(content.contains("\"moduleResolution\": \"bundler\""));
    assert!(content.contains("\"@/*\": [\"./src/*\"]"));
}

#[test]
fn test_client_react_vite_config_content() {
    let (tmp, _) = render_client("react");
    let content = read_output(&tmp, "vite.config.ts");

    assert!(content.contains("import react from '@vitejs/plugin-react'"));
    assert!(content.contains("import tailwindcss from '@tailwindcss/vite'"));
    assert!(content.contains("port: 3000"));
    assert!(content.contains("'/api'"));
}

#[test]
fn test_client_react_app_tsx_content() {
    let (tmp, _) = render_client("react");
    let content = read_output(&tmp, "src/app/App.tsx");

    assert!(content.contains("QueryClientProvider"));
    assert!(content.contains("RouterProvider"));
    assert!(content.contains("export function App()"));
}

#[test]
fn test_client_react_no_tera_syntax() {
    let (tmp, names) = render_client("react");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {}", name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

#[test]
fn test_client_react_service_name_substitution() {
    let (tmp, _) = render_client("react");
    let content = read_output(&tmp, "package.json");

    // service_name が正しく "test-app" に置換されている
    assert!(content.contains("\"name\": \"test-app\""));
    // Tera 変数が残っていない
    assert!(!content.contains("service_name"));
}

#[test]
fn test_client_react_test_files_exist() {
    let (_, names) = render_client("react");

    assert!(
        names.iter().any(|n| n == "tests/App.test.tsx"),
        "tests/App.test.tsx missing"
    );
    assert!(
        names.iter().any(|n| n == "tests/testutil/setup.ts"),
        "tests/testutil/setup.ts missing"
    );
    assert!(
        names.iter().any(|n| n == "tests/testutil/msw-setup.ts"),
        "tests/testutil/msw-setup.ts missing"
    );
}

#[test]
fn test_client_react_eslint_config() {
    let (tmp, names) = render_client("react");

    assert!(
        names.iter().any(|n| n == "eslint.config.mjs"),
        "eslint.config.mjs missing"
    );

    let content = read_output(&tmp, "eslint.config.mjs");
    assert!(content.contains("typescript-eslint"));
    assert!(content.contains("react-hooks"));
}

#[test]
fn test_client_react_prettier_config() {
    let (tmp, names) = render_client("react");

    assert!(
        names.iter().any(|n| n == ".prettierrc"),
        ".prettierrc missing"
    );

    let content = read_output(&tmp, ".prettierrc");
    assert!(content.contains("\"singleQuote\": true"));
    assert!(content.contains("\"trailingComma\": \"all\""));
}

// =========================================================================
// Flutter クライアント
// =========================================================================

#[test]
fn test_client_flutter_file_list() {
    let (_, names) = render_client("flutter");

    assert!(names.iter().any(|n| n == "pubspec.yaml"), "pubspec.yaml missing");
    assert!(names.iter().any(|n| n == "analysis_options.yaml"), "analysis_options.yaml missing");
    assert!(names.iter().any(|n| n == "lib/main.dart"), "lib/main.dart missing");
    assert!(names.iter().any(|n| n == "lib/app/router.dart"), "lib/app/router.dart missing");
    assert!(names.iter().any(|n| n == "lib/utils/dio_client.dart"), "lib/utils/dio_client.dart missing");
    assert!(names.iter().any(|n| n == "Dockerfile"), "Dockerfile missing");
    assert!(names.iter().any(|n| n == "nginx.conf"), "nginx.conf missing");
    assert!(names.iter().any(|n| n == "test/widget_test.dart"), "test/widget_test.dart missing");
    assert!(names.iter().any(|n| n == "README.md"), "README.md missing");
}

#[test]
fn test_client_flutter_pubspec_content() {
    let (tmp, _) = render_client("flutter");
    let content = read_output(&tmp, "pubspec.yaml");

    assert!(content.contains("name: test_app"));
    assert!(content.contains("flutter_riverpod:"));
    assert!(content.contains("go_router:"));
    assert!(content.contains("dio:"));
    assert!(content.contains("freezed_annotation:"));
}

#[test]
fn test_client_flutter_main_dart_content() {
    let (tmp, _) = render_client("flutter");
    let content = read_output(&tmp, "lib/main.dart");

    assert!(content.contains("import 'package:flutter/material.dart';"));
    assert!(content.contains("import 'package:flutter_riverpod/flutter_riverpod.dart';"));
    assert!(content.contains("import 'package:test_app/app/router.dart';"));
    assert!(content.contains("ProviderScope"));
    assert!(content.contains("MaterialApp.router"));
    assert!(content.contains("title: 'TestApp'"));
}

#[test]
fn test_client_flutter_no_tera_syntax() {
    let (tmp, names) = render_client("flutter");

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {}", name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

#[test]
fn test_client_flutter_service_name_substitution() {
    let (tmp, _) = render_client("flutter");
    let content = read_output(&tmp, "pubspec.yaml");

    // service_name_snake が正しく "test_app" に置換されている
    assert!(content.contains("name: test_app"));

    let main = read_output(&tmp, "lib/main.dart");
    // service_name_pascal が正しく "TestApp" に置換されている
    assert!(main.contains("title: 'TestApp'"));
}
