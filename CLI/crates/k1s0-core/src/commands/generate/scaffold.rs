use anyhow::Result;
use std::fs;
use std::path::Path;

use super::types::{ApiStyle, Framework, GenerateConfig, LangFw, Language};

/// サーバーひな形を生成する。
pub(super) fn generate_server(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let LangFw::Language(lang) = config.lang_fw else {
        unreachable!()
    };
    let service_name = config.detail.name.as_deref().unwrap_or("service");

    match lang {
        Language::Go => generate_go_server(output_path, service_name, config)?,
        Language::Rust => generate_rust_server(output_path, service_name, config)?,
        _ => unreachable!("サーバーの言語は Go/Rust のみ"),
    }

    Ok(())
}

fn generate_go_server(
    output_path: &Path,
    service_name: &str,
    config: &GenerateConfig,
) -> Result<()> {
    // cmd/
    let cmd_dir = output_path.join("cmd");
    fs::create_dir_all(&cmd_dir)?;
    fs::write(
        cmd_dir.join("main.go"),
        format!(
            r#"package main

import "fmt"

func main() {{
	fmt.Println("Starting {service_name} server...")
}}
"#
        ),
    )?;

    // internal/
    let internal_dir = output_path.join("internal");
    fs::create_dir_all(internal_dir.join("handler"))?;
    fs::create_dir_all(internal_dir.join("service"))?;
    fs::create_dir_all(internal_dir.join("repository"))?;

    fs::write(internal_dir.join("handler/handler.go"), "package handler\n")?;
    fs::write(internal_dir.join("service/service.go"), "package service\n")?;
    fs::write(
        internal_dir.join("repository/repository.go"),
        "package repository\n",
    )?;

    // go.mod
    fs::write(
        output_path.join("go.mod"),
        format!("module {service_name}\n\ngo 1.21\n"),
    )?;

    // Dockerfile
    fs::write(
        output_path.join("Dockerfile"),
        generate_go_dockerfile(service_name),
    )?;

    // API定義
    for api in &config.detail.api_styles {
        match api {
            ApiStyle::Rest => {
                let api_dir = output_path.join("api/openapi");
                fs::create_dir_all(&api_dir)?;
                fs::write(
                    api_dir.join("openapi.yaml"),
                    generate_openapi_stub(service_name),
                )?;
            }
            ApiStyle::Grpc => {
                let proto_dir = output_path.join("api/proto");
                fs::create_dir_all(&proto_dir)?;
                fs::write(
                    proto_dir.join(format!("{service_name}.proto")),
                    generate_proto_stub(service_name),
                )?;
            }
            ApiStyle::GraphQL => {
                let gql_dir = output_path.join("api/graphql");
                fs::create_dir_all(&gql_dir)?;
                fs::write(
                    gql_dir.join("schema.graphql"),
                    generate_graphql_stub(service_name),
                )?;
            }
        }
    }

    Ok(())
}

fn generate_rust_server(
    output_path: &Path,
    service_name: &str,
    config: &GenerateConfig,
) -> Result<()> {
    // src/
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        src_dir.join("main.rs"),
        format!(
            r#"fn main() {{
    println!("Starting {service_name} server...");
}}
"#
        ),
    )?;

    // Cargo.toml
    fs::write(
        output_path.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{service_name}"
version = "0.1.0"
edition = "2021"
"#
        ),
    )?;

    // Dockerfile
    fs::write(
        output_path.join("Dockerfile"),
        generate_rust_dockerfile(service_name),
    )?;

    // API定義
    for api in &config.detail.api_styles {
        match api {
            ApiStyle::Rest => {
                let api_dir = output_path.join("api/openapi");
                fs::create_dir_all(&api_dir)?;
                fs::write(
                    api_dir.join("openapi.yaml"),
                    generate_openapi_stub(service_name),
                )?;
            }
            ApiStyle::Grpc => {
                let proto_dir = output_path.join("api/proto");
                fs::create_dir_all(&proto_dir)?;
                fs::write(
                    proto_dir.join(format!("{service_name}.proto")),
                    generate_proto_stub(service_name),
                )?;
            }
            ApiStyle::GraphQL => {
                let gql_dir = output_path.join("api/graphql");
                fs::create_dir_all(&gql_dir)?;
                fs::write(
                    gql_dir.join("schema.graphql"),
                    generate_graphql_stub(service_name),
                )?;
            }
        }
    }

    Ok(())
}

/// クライアントひな形を生成する。
pub(super) fn generate_client(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let LangFw::Framework(fw) = config.lang_fw else {
        unreachable!()
    };
    let app_name = config.detail.name.as_deref().unwrap_or("app");

    match fw {
        Framework::React => generate_react_client(output_path, app_name)?,
        Framework::Flutter => generate_flutter_client(output_path, app_name)?,
    }

    Ok(())
}

fn generate_react_client(output_path: &Path, app_name: &str) -> Result<()> {
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir)?;

    fs::write(
        output_path.join("package.json"),
        format!(
            r#"{{
  "name": "{app_name}",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "test": "vitest"
  }}
}}
"#
        ),
    )?;

    fs::write(
        src_dir.join("App.tsx"),
        format!(
            r"function App() {{
  return <div>{app_name}</div>;
}}

export default App;
"
        ),
    )?;

    fs::write(
        src_dir.join("main.tsx"),
        r#"import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
"#,
    )?;

    fs::write(
        output_path.join("index.html"),
        format!(
            r#"<!DOCTYPE html>
<html lang="ja">
<head><meta charset="UTF-8"><title>{app_name}</title></head>
<body><div id="root"></div><script type="module" src="/src/main.tsx"></script></body>
</html>
"#
        ),
    )?;

    // service Tier の場合: config-schema.yaml と初期型定義ファイルを生成
    generate_config_schema_stub(output_path, app_name, "react")?;

    // navigation.yaml スタブとプレースホルダーを生成
    generate_navigation_stub(output_path, app_name, "react")?;

    Ok(())
}

/// service Tier クライアント向けの config-schema.yaml スタブを生成する。
/// scaffold 内ではTier情報を持たないため、呼び出し元で制御する代わりに
/// 常に生成して利用者に活用してもらう方式とする。
fn generate_config_schema_stub(output_path: &Path, app_name: &str, framework: &str) -> Result<()> {
    let snake_name = app_name.replace('-', "_");
    fs::write(
        output_path.join("config-schema.yaml"),
        format!(
            r#"# $schema: ./config-schema-schema.json
version: 1

service: {app_name}
namespace_prefix: service.{snake_name}

categories:
  - id: general
    label: 一般設定
    icon: settings
    namespaces:
      - service.{snake_name}.general
    fields:
      - key: example_flag
        label: サンプル機能フラグ
        description: この設定はサンプルです。削除して独自の設定を追加してください
        type: boolean
        default: false
"#
        ),
    )?;

    match framework {
        "react" => {
            let gen_dir = output_path.join("src/config/__generated__");
            fs::create_dir_all(&gen_dir)?;
            fs::write(
                gen_dir.join("config-types.ts"),
                "// src/config/__generated__/config-types.ts\n\
                 // このファイルは CLI が自動生成する。直接編集しないこと。\n\
                 // k1s0 generate config-types で再生成できます。\n\n\
                 // TODO: k1s0 generate config-types を実行して型定義を生成してください。\n\
                 export const ConfigKeys = {} as const;\n\
                 export type ConfigValues = Record<string, unknown>;\n",
            )?;
        }
        "flutter" => {
            let gen_dir = output_path.join("lib/config/__generated__");
            fs::create_dir_all(&gen_dir)?;
            fs::write(
                gen_dir.join("config_types.dart"),
                "// lib/config/__generated__/config_types.dart\n\
                 // このファイルは CLI が自動生成する。直接編集しないこと。\n\n\
                 // TODO: k1s0 generate config-types を実行して型定義を生成してください。\n",
            )?;
        }
        _ => {}
    }

    Ok(())
}

fn generate_flutter_client(output_path: &Path, app_name: &str) -> Result<()> {
    let lib_dir = output_path.join("lib");
    fs::create_dir_all(&lib_dir)?;

    fs::write(
        output_path.join("pubspec.yaml"),
        format!(
            r#"name: {app_name}
description: A Flutter application
version: 0.1.0

environment:
  sdk: ">=3.0.0 <4.0.0"

dependencies:
  flutter:
    sdk: flutter
"#
        ),
    )?;

    fs::write(
        lib_dir.join("main.dart"),
        format!(
            r"import 'package:flutter/material.dart';

void main() {{
  runApp(const MyApp());
}}

class MyApp extends StatelessWidget {{
  const MyApp({{super.key}});

  @override
  Widget build(BuildContext context) {{
    return MaterialApp(
      title: '{app_name}',
      home: const Scaffold(
        body: Center(child: Text('{app_name}')),
      ),
    );
  }}
}}
"
        ),
    )?;

    // service Tier の場合: config-schema.yaml と初期型定義ファイルを生成
    generate_config_schema_stub(output_path, app_name, "flutter")?;

    // navigation.yaml スタブとプレースホルダーを生成
    generate_navigation_stub(output_path, app_name, "flutter")?;

    Ok(())
}

/// ライブラリひな形を生成する。
#[allow(clippy::too_many_lines)]
pub(super) fn generate_library(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let LangFw::Language(lang) = config.lang_fw else {
        unreachable!()
    };
    let lib_name = config.detail.name.as_deref().unwrap_or("lib");

    match lang {
        Language::Go => {
            fs::write(
                output_path.join("go.mod"),
                format!("module {lib_name}\n\ngo 1.21\n"),
            )?;
            fs::write(
                output_path.join(format!("{}.go", lib_name.replace('-', "_"))),
                format!("package {}\n", lib_name.replace('-', "_")),
            )?;
            fs::write(
                output_path.join(format!("{}_test.go", lib_name.replace('-', "_"))),
                format!(
                    r#"package {}

import "testing"

func TestPlaceholder(t *testing.T) {{
	// TODO: implement
}}
"#,
                    lib_name.replace('-', "_")
                ),
            )?;
        }
        Language::Rust => {
            fs::write(
                output_path.join("Cargo.toml"),
                format!(
                    r#"[package]
name = "{lib_name}"
version = "0.1.0"
edition = "2021"

[lib]
"#
                ),
            )?;
            let src_dir = output_path.join("src");
            fs::create_dir_all(&src_dir)?;
            fs::write(
                src_dir.join("lib.rs"),
                r"#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
",
            )?;
        }
        Language::TypeScript => {
            fs::write(
                output_path.join("package.json"),
                format!(
                    r#"{{
  "name": "{lib_name}",
  "version": "0.1.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {{
    "build": "tsc",
    "test": "vitest"
  }}
}}
"#
                ),
            )?;
            let src_dir = output_path.join("src");
            fs::create_dir_all(&src_dir)?;
            fs::write(src_dir.join("index.ts"), "export {};\n")?;
            fs::write(
                output_path.join("tsconfig.json"),
                r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "declaration": true,
    "outDir": "dist",
    "strict": true
  },
  "include": ["src"]
}
"#,
            )?;
        }
        Language::Dart => {
            fs::write(
                output_path.join("pubspec.yaml"),
                format!(
                    r#"name: {lib_name}
version: 0.1.0

environment:
  sdk: ">=3.0.0 <4.0.0"
"#
                ),
            )?;
            let lib_dir = output_path.join("lib");
            fs::create_dir_all(&lib_dir)?;
            fs::write(
                lib_dir.join(format!("{}.dart", lib_name.replace('-', "_"))),
                format!("library {};\n", lib_name.replace('-', "_")),
            )?;
        }
        Language::Python => {
            let snake_name = lib_name.replace('-', "_");
            let pascal_name = snake_name
                .split('_')
                .map(|s| {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<String>();

            fs::write(
                output_path.join("pyproject.toml"),
                format!(
                    r#"[project]
name = "k1s0-{lib_name}"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = []

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/k1s0_{snake_name}"]

[tool.pytest.ini_options]
asyncio_mode = "auto"
testpaths = ["tests"]

[tool.coverage.run]
source = ["src"]
branch = true

[tool.coverage.report]
fail_under = 85
"#
                ),
            )?;

            let src_pkg_dir = output_path.join("src").join(format!("k1s0_{snake_name}"));
            fs::create_dir_all(&src_pkg_dir)?;

            fs::write(
                src_pkg_dir.join("__init__.py"),
                format!(
                    r#""""k1s0-{lib_name} ライブラリ"""
from .{snake_name} import Client, Config
from .exceptions import {pascal_name}Error

__all__ = ["Client", "Config", "{pascal_name}Error"]
"#
                ),
            )?;

            fs::write(
                src_pkg_dir.join("exceptions.py"),
                format!(
                    r#""""{lib_name} ライブラリの例外型定義"""
from __future__ import annotations


class {pascal_name}Error(Exception):
    """{pascal_name}ライブラリのエラー基底クラス。"""

    def __init__(
        self,
        code: str,
        message: str,
        cause: Exception | None = None,
    ) -> None:
        super().__init__(message)
        self.code = code
        if cause is not None:
            self.__cause__ = cause

    def __str__(self) -> str:
        return f"{{self.code}}: {{super().__str__()}}"
"#
                ),
            )?;

            fs::write(
                src_pkg_dir.join(format!("{snake_name}.py")),
                format!(
                    r#""""{lib_name} ライブラリの実装"""
from __future__ import annotations

from .exceptions import {pascal_name}Error


class Config:
    """ライブラリの設定クラス。"""

    def __init__(self, name: str) -> None:
        self.name = name

    def validate(self) -> None:
        """バリデーションを行う。"""
        if not self.name:
            raise {pascal_name}Error(
                code="INVALID_CONFIG",
                message="name is required",
            )


class Client:
    """{lib_name} クライアント。"""

    def __init__(self, config: Config) -> None:
        self._config = config

    @property
    def name(self) -> str:
        """設定名を返す。"""
        return self._config.name
"#
                ),
            )?;

            let tests_dir = output_path.join("tests");
            fs::create_dir_all(&tests_dir)?;
            fs::write(tests_dir.join("__init__.py"), "")?;

            fs::write(
                tests_dir.join(format!("test_{snake_name}.py")),
                format!(
                    r#""""{lib_name} ライブラリのユニットテスト"""
import pytest
from k1s0_{snake_name}.{snake_name} import Client, Config
from k1s0_{snake_name}.exceptions import {pascal_name}Error


def test_client_name() -> None:
    config = Config(name="test")
    client = Client(config=config)
    assert client.name == "test"


def test_validate_ok() -> None:
    config = Config(name="test")
    config.validate()  # should not raise


def test_validate_error() -> None:
    config = Config(name="")
    with pytest.raises({pascal_name}Error) as exc_info:
        config.validate()
    assert exc_info.value.code == "INVALID_CONFIG"
"#
                ),
            )?;

            fs::write(
                output_path.join("README.md"),
                format!("# k1s0-{lib_name}\n\nk1s0 {lib_name} Python ライブラリ\n"),
            )?;
        }
        Language::Swift => {
            let snake_name = lib_name.replace('-', "_");
            fs::write(
                output_path.join("Package.swift"),
                format!(
                    r#"// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "k1s0-{}",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "K1s0{}", targets: ["K1s0{}"]),
    ],
    targets: [
        .target(name: "K1s0{}", path: "Sources/{}"),
        .testTarget(name: "K1s0{}Tests", dependencies: ["K1s0{}"], path: "Tests/{}_tests"),
    ]
)
"#,
                    lib_name,
                    snake_name
                        .replace('_', "")
                        .chars()
                        .enumerate()
                        .map(|(i, c)| if i == 0 {
                            c.to_uppercase().next().unwrap_or(c)
                        } else {
                            c
                        })
                        .collect::<String>(),
                    snake_name
                        .replace('_', "")
                        .chars()
                        .enumerate()
                        .map(|(i, c)| if i == 0 {
                            c.to_uppercase().next().unwrap_or(c)
                        } else {
                            c
                        })
                        .collect::<String>(),
                    snake_name
                        .replace('_', "")
                        .chars()
                        .enumerate()
                        .map(|(i, c)| if i == 0 {
                            c.to_uppercase().next().unwrap_or(c)
                        } else {
                            c
                        })
                        .collect::<String>(),
                    snake_name,
                    snake_name
                        .replace('_', "")
                        .chars()
                        .enumerate()
                        .map(|(i, c)| if i == 0 {
                            c.to_uppercase().next().unwrap_or(c)
                        } else {
                            c
                        })
                        .collect::<String>(),
                    snake_name
                        .replace('_', "")
                        .chars()
                        .enumerate()
                        .map(|(i, c)| if i == 0 {
                            c.to_uppercase().next().unwrap_or(c)
                        } else {
                            c
                        })
                        .collect::<String>(),
                    snake_name,
                ),
            )?;
            let src_dir = output_path.join("Sources").join(&snake_name);
            fs::create_dir_all(&src_dir)?;
            fs::write(src_dir.join("Client.swift"), "// TODO: implement\n")?;
            let test_dir = output_path
                .join("Tests")
                .join(format!("{snake_name}_tests"));
            fs::create_dir_all(&test_dir)?;
            fs::write(
                test_dir.join("ClientTests.swift"),
                "// TODO: implement tests\n",
            )?;
        }
    }

    Ok(())
}

/// データベースひな形を生成する。
pub(super) fn generate_database(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let (db_name, rdbms) = match &config.lang_fw {
        LangFw::Database { name, rdbms } => (name.as_str(), *rdbms),
        _ => unreachable!(),
    };

    let migrations_dir = output_path.join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    // D-11: seeds/ と schema/ ディレクトリを作成
    let seeds_dir = output_path.join("seeds");
    fs::create_dir_all(&seeds_dir)?;
    let schema_dir = output_path.join("schema");
    fs::create_dir_all(&schema_dir)?;

    // D-12: 3桁プレフィックスに修正 (000001_init -> 001_init)
    fs::write(
        migrations_dir.join("001_init.up.sql"),
        format!(
            "-- {} の初期マイグレーション ({})\n-- TODO: テーブル定義を追加\n",
            db_name,
            rdbms.as_str()
        ),
    )?;

    fs::write(
        migrations_dir.join("001_init.down.sql"),
        "-- ロールバック\n-- TODO: DROP TABLE 文を追加\n",
    )?;

    // 設定ファイル
    fs::write(
        output_path.join("database.yaml"),
        format!(
            r"name: {}
rdbms: {}
",
            db_name,
            rdbms.as_str()
        ),
    )?;

    Ok(())
}

/// navigation.yaml スタブとフレームワーク別のプレースホルダーを生成する。
fn generate_navigation_stub(output_path: &Path, _app_name: &str, framework: &str) -> Result<()> {
    // navigation.yaml (React: public/, Flutter: assets/)
    let nav_yaml = format!(
        r#"# $schema: ./navigation-schema.json
version: 1

guards:
  - id: auth_required
    type: auth_required
    redirect_to: /login

routes:
  - id: root
    path: /
    redirect_to: /dashboard

  - id: login
    path: /login
    component_id: LoginPage
    guards: []

  - id: dashboard
    path: /dashboard
    component_id: DashboardPage
    guards: [auth_required]
    transition: fade
"#
    );

    match framework {
        "react" => {
            let public_dir = output_path.join("public");
            fs::create_dir_all(&public_dir)?;
            fs::write(public_dir.join("navigation.yaml"), &nav_yaml)?;

            // component-registry.ts プレースホルダー
            let nav_gen_dir = output_path.join("src/navigation/__generated__");
            fs::create_dir_all(&nav_gen_dir)?;
            fs::write(
                nav_gen_dir.join("component-registry.ts"),
                "// src/navigation/__generated__/component-registry.ts\n\
                 // このファイルは雛形です。navigation.yaml の component_id に対応するコンポーネントを登録してください。\n\n\
                 import type { ComponentRegistry } from 'system-client';\n\n\
                 export const componentRegistry: ComponentRegistry = {\n\
                 \x20 LoginPage:     () => import('../../pages/LoginPage'),\n\
                 \x20 DashboardPage: () => import('../../pages/DashboardPage'),\n\
                 };\n",
            )?;

            // route-types.ts プレースホルダー
            fs::write(
                nav_gen_dir.join("route-types.ts"),
                "// src/navigation/__generated__/route-types.ts\n\
                 // このファイルは CLI が自動生成する。直接編集しないこと。\n\
                 // k1s0 generate navigation で再生成できます。\n\n\
                 // TODO: k1s0 generate navigation --target react を実行して型定義を生成してください。\n\
                 export const RouteIds = {} as const;\n\
                 export type RouteId = string;\n\
                 export type RouteParams = Record<string, Record<string, never>>;\n",
            )?;
        }
        "flutter" => {
            let assets_dir = output_path.join("assets");
            fs::create_dir_all(&assets_dir)?;
            fs::write(assets_dir.join("navigation.yaml"), &nav_yaml)?;

            // route_ids.dart プレースホルダー
            let nav_gen_dir = output_path.join("lib/navigation/__generated__");
            fs::create_dir_all(&nav_gen_dir)?;
            fs::write(
                nav_gen_dir.join("route_ids.dart"),
                "// lib/navigation/__generated__/route_ids.dart\n\
                 // このファイルは CLI が自動生成する。直接編集しないこと。\n\n\
                 // TODO: k1s0 generate navigation --target flutter を実行して型定義を生成してください。\n",
            )?;
        }
        _ => {}
    }

    Ok(())
}

// --- テンプレートスタブ生成 ---

fn generate_go_dockerfile(service_name: &str) -> String {
    format!(
        r#"FROM golang:1.21-alpine AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /bin/{service_name} ./cmd/

FROM alpine:3.19
COPY --from=builder /bin/{service_name} /bin/{service_name}
ENTRYPOINT ["/bin/{service_name}"]
"#
    )
}

fn generate_rust_dockerfile(service_name: &str) -> String {
    format!(
        r#"FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/{service_name} /usr/local/bin/{service_name}
ENTRYPOINT ["{service_name}"]
"#
    )
}

pub(super) fn generate_openapi_stub(service_name: &str) -> String {
    format!(
        r#"openapi: "3.0.3"
info:
  title: {service_name} API
  version: "0.1.0"
paths: {{}}
"#
    )
}

pub(super) fn generate_proto_stub(service_name: &str) -> String {
    let pkg = service_name.replace('-', "_");
    format!(
        r#"syntax = "proto3";

package {pkg};

service {pkg}Service {{
  // TODO: RPC メソッドを定義
}}
"#
    )
}

pub(super) fn generate_graphql_stub(service_name: &str) -> String {
    format!(
        r"# {service_name} GraphQL Schema

type Query {{
  hello: String!
}}
"
    )
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_graphql_stub() {
        let gql = generate_graphql_stub("order");
        assert!(gql.contains("order GraphQL Schema"));
        assert!(gql.contains("type Query"));
    }

    #[test]
    fn test_generate_openapi_stub() {
        let yaml = generate_openapi_stub("auth");
        assert!(yaml.contains("auth API"));
        assert!(yaml.contains("openapi:"));
    }

    #[test]
    fn test_generate_proto_stub() {
        let proto = generate_proto_stub("order-api");
        assert!(proto.contains("package order_api"));
        assert!(proto.contains("service order_apiService"));
    }
}
