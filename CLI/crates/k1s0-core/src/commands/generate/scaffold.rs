use anyhow::Result;
use std::fs;
use std::path::Path;

use super::types::{ApiStyle, Framework, GenerateConfig, LangFw, Language};

/// サーバーひな形を生成する。
pub(super) fn generate_server(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let lang = match config.lang_fw {
        LangFw::Language(l) => l,
        _ => unreachable!(),
    };
    let service_name = config.detail.name.as_deref().unwrap_or("service");

    match lang {
        Language::Go => generate_go_server(output_path, service_name, config)?,
        Language::Rust => generate_rust_server(output_path, service_name, config)?,
        _ => unreachable!("サーバーの言語は Go/Rust のみ"),
    }

    Ok(())
}

fn generate_go_server(output_path: &Path, service_name: &str, config: &GenerateConfig) -> Result<()> {
    // cmd/
    let cmd_dir = output_path.join("cmd");
    fs::create_dir_all(&cmd_dir)?;
    fs::write(
        cmd_dir.join("main.go"),
        format!(
            r#"package main

import "fmt"

func main() {{
	fmt.Println("Starting {} server...")
}}
"#,
            service_name
        ),
    )?;

    // internal/
    let internal_dir = output_path.join("internal");
    fs::create_dir_all(internal_dir.join("handler"))?;
    fs::create_dir_all(internal_dir.join("service"))?;
    fs::create_dir_all(internal_dir.join("repository"))?;

    fs::write(
        internal_dir.join("handler/handler.go"),
        "package handler\n",
    )?;
    fs::write(
        internal_dir.join("service/service.go"),
        "package service\n",
    )?;
    fs::write(
        internal_dir.join("repository/repository.go"),
        "package repository\n",
    )?;

    // go.mod
    fs::write(
        output_path.join("go.mod"),
        format!("module {}\n\ngo 1.21\n", service_name),
    )?;

    // Dockerfile
    fs::write(output_path.join("Dockerfile"), generate_go_dockerfile(service_name))?;

    // API定義
    for api in &config.detail.api_styles {
        match api {
            ApiStyle::Rest => {
                let api_dir = output_path.join("api/openapi");
                fs::create_dir_all(&api_dir)?;
                fs::write(api_dir.join("openapi.yaml"), generate_openapi_stub(service_name))?;
            }
            ApiStyle::Grpc => {
                let proto_dir = output_path.join("api/proto");
                fs::create_dir_all(&proto_dir)?;
                fs::write(
                    proto_dir.join(format!("{}.proto", service_name)),
                    generate_proto_stub(service_name),
                )?;
            }
            ApiStyle::GraphQL => {
                let gql_dir = output_path.join("api/graphql");
                fs::create_dir_all(&gql_dir)?;
                fs::write(gql_dir.join("schema.graphql"), generate_graphql_stub(service_name))?;
            }
        }
    }

    Ok(())
}

fn generate_rust_server(output_path: &Path, service_name: &str, config: &GenerateConfig) -> Result<()> {
    // src/
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        src_dir.join("main.rs"),
        format!(
            r#"fn main() {{
    println!("Starting {} server...");
}}
"#,
            service_name
        ),
    )?;

    // Cargo.toml
    fs::write(
        output_path.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#,
            service_name
        ),
    )?;

    // Dockerfile
    fs::write(output_path.join("Dockerfile"), generate_rust_dockerfile(service_name))?;

    // API定義
    for api in &config.detail.api_styles {
        match api {
            ApiStyle::Rest => {
                let api_dir = output_path.join("api/openapi");
                fs::create_dir_all(&api_dir)?;
                fs::write(api_dir.join("openapi.yaml"), generate_openapi_stub(service_name))?;
            }
            ApiStyle::Grpc => {
                let proto_dir = output_path.join("api/proto");
                fs::create_dir_all(&proto_dir)?;
                fs::write(
                    proto_dir.join(format!("{}.proto", service_name)),
                    generate_proto_stub(service_name),
                )?;
            }
            ApiStyle::GraphQL => {
                let gql_dir = output_path.join("api/graphql");
                fs::create_dir_all(&gql_dir)?;
                fs::write(gql_dir.join("schema.graphql"), generate_graphql_stub(service_name))?;
            }
        }
    }

    Ok(())
}

/// クライアントひな形を生成する。
pub(super) fn generate_client(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let fw = match config.lang_fw {
        LangFw::Framework(f) => f,
        _ => unreachable!(),
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
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "test": "vitest"
  }}
}}
"#,
            app_name
        ),
    )?;

    fs::write(
        src_dir.join("App.tsx"),
        format!(
            r#"function App() {{
  return <div>{}</div>;
}}

export default App;
"#,
            app_name
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

    fs::write(output_path.join("index.html"), format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head><meta charset="UTF-8"><title>{}</title></head>
<body><div id="root"></div><script type="module" src="/src/main.tsx"></script></body>
</html>
"#, app_name))?;

    Ok(())
}

fn generate_flutter_client(output_path: &Path, app_name: &str) -> Result<()> {
    let lib_dir = output_path.join("lib");
    fs::create_dir_all(&lib_dir)?;

    fs::write(
        output_path.join("pubspec.yaml"),
        format!(
            r#"name: {}
description: A Flutter application
version: 0.1.0

environment:
  sdk: ">=3.0.0 <4.0.0"

dependencies:
  flutter:
    sdk: flutter
"#,
            app_name
        ),
    )?;

    fs::write(
        lib_dir.join("main.dart"),
        format!(
            r#"import 'package:flutter/material.dart';

void main() {{
  runApp(const MyApp());
}}

class MyApp extends StatelessWidget {{
  const MyApp({{super.key}});

  @override
  Widget build(BuildContext context) {{
    return MaterialApp(
      title: '{}',
      home: const Scaffold(
        body: Center(child: Text('{}')),
      ),
    );
  }}
}}
"#,
            app_name, app_name
        ),
    )?;

    Ok(())
}

/// ライブラリひな形を生成する。
pub(super) fn generate_library(config: &GenerateConfig, output_path: &Path) -> Result<()> {
    let lang = match config.lang_fw {
        LangFw::Language(l) => l,
        _ => unreachable!(),
    };
    let lib_name = config.detail.name.as_deref().unwrap_or("lib");

    match lang {
        Language::Go => {
            fs::write(
                output_path.join("go.mod"),
                format!("module {}\n\ngo 1.21\n", lib_name),
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
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
"#,
                    lib_name
                ),
            )?;
            let src_dir = output_path.join("src");
            fs::create_dir_all(&src_dir)?;
            fs::write(
                src_dir.join("lib.rs"),
                r#"#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
            )?;
        }
        Language::TypeScript => {
            fs::write(
                output_path.join("package.json"),
                format!(
                    r#"{{
  "name": "{}",
  "version": "0.1.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {{
    "build": "tsc",
    "test": "vitest"
  }}
}}
"#,
                    lib_name
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
                    r#"name: {}
version: 0.1.0

environment:
  sdk: ">=3.0.0 <4.0.0"
"#,
                    lib_name
                ),
            )?;
            let lib_dir = output_path.join("lib");
            fs::create_dir_all(&lib_dir)?;
            fs::write(
                lib_dir.join(format!("{}.dart", lib_name.replace('-', "_"))),
                format!("library {};\n", lib_name.replace('-', "_")),
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
            r#"name: {}
rdbms: {}
"#,
            db_name,
            rdbms.as_str()
        ),
    )?;

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
RUN CGO_ENABLED=0 go build -o /bin/{} ./cmd/

FROM alpine:3.19
COPY --from=builder /bin/{} /bin/{}
ENTRYPOINT ["/bin/{}"]
"#,
        service_name, service_name, service_name, service_name
    )
}

fn generate_rust_dockerfile(service_name: &str) -> String {
    format!(
        r#"FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/{} /usr/local/bin/{}
ENTRYPOINT ["{}"]
"#,
        service_name, service_name, service_name
    )
}

pub(super) fn generate_openapi_stub(service_name: &str) -> String {
    format!(
        r#"openapi: "3.0.3"
info:
  title: {} API
  version: "0.1.0"
paths: {{}}
"#,
        service_name
    )
}

pub(super) fn generate_proto_stub(service_name: &str) -> String {
    let pkg = service_name.replace('-', "_");
    format!(
        r#"syntax = "proto3";

package {};

service {}Service {{
  // TODO: RPC メソッドを定義
}}
"#,
        pkg, pkg
    )
}

pub(super) fn generate_graphql_stub(service_name: &str) -> String {
    format!(
        r#"# {} GraphQL Schema

type Query {{
  hello: String!
}}
"#,
        service_name
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
