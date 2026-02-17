pub mod context;
pub mod filters;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as AnyhowContext, Result};
use tera::Tera;
use walkdir::WalkDir;

use context::TemplateContext;

/// テンプレートエンジンを初期化する。
///
/// 指定されたディレクトリからテンプレートファイルを読み込み、
/// カスタムフィルタを登録した Tera インスタンスを返す。
#[allow(dead_code)]
pub fn create_engine(templates_dir: &str) -> Result<Tera> {
    let glob_pattern = format!("{}/**/*", templates_dir);
    let mut tera = Tera::new(&glob_pattern)?;
    filters::register_filters(&mut tera);
    Ok(tera)
}

/// テンプレートをレンダリングする。
///
/// テンプレート名と Tera コンテキストを受け取り、
/// レンダリング結果の文字列を返す。
#[allow(dead_code)]
pub fn render(engine: &Tera, template_name: &str, ctx: &tera::Context) -> Result<String> {
    let rendered = engine.render(template_name, ctx)?;
    Ok(rendered)
}

/// テンプレートエンジン。
///
/// Tera エンジンをラップし、CLI/templates/ から .tera ファイルを読み込み、
/// TemplateContext を適用してレンダリングする。
pub struct TemplateEngine {
    pub(crate) tera: Tera,
    template_dir: PathBuf,
}

/// テンプレートファイルが条件に合致するかを判定するための情報。
struct TemplateFileInfo {
    /// テンプレートディレクトリからの相対パス (例: "cmd/main.go.tera")
    relative_path: PathBuf,
    /// Tera に登録するテンプレート名
    template_name: String,
}

impl TemplateEngine {
    /// テンプレートエンジンを初期化する。
    ///
    /// 指定されたディレクトリからテンプレートファイルを読み込み、
    /// カスタムフィルタを登録した状態で返す。
    ///
    /// # Arguments
    /// * `template_dir` - テンプレートファイルのルートディレクトリ (例: "CLI/templates")
    pub fn new(template_dir: &Path) -> Result<Self> {
        let mut tera = Tera::default();
        filters::register_filters(&mut tera);

        Ok(Self {
            tera,
            template_dir: template_dir.to_path_buf(),
        })
    }

    /// コンテキストに基づいてテンプレートをレンダリングし、出力先に書き込む。
    ///
    /// 処理の流れ:
    /// 1. kind + language に対応するテンプレートディレクトリを選択
    /// 2. 条件付きファイル (api_style, has_database 等) をフィルタ
    /// 3. ファイル名のプレースホルダ ({name}, {module} 等) を置換
    /// 4. 各テンプレートをレンダリングして出力先に書き込み
    ///
    /// # Arguments
    /// * `ctx` - テンプレートコンテキスト
    /// * `output_dir` - 生成先のディレクトリ
    ///
    /// # Returns
    /// 生成されたファイルのパス一覧
    pub fn render_to_dir(
        &mut self,
        ctx: &TemplateContext,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let tera_ctx = ctx.to_tera_context();

        // kind + language に対応するテンプレートディレクトリを決定
        // CICD・Helm・Terraform・Docker Compose・devcontainer・Service Mesh テンプレートは
        // 言語サブディレクトリを持たないフラット構造
        let flat_kinds = [
            "cicd",
            "helm",
            "terraform",
            "docker-compose",
            "devcontainer",
            "service-mesh",
            "kong",
            "keycloak",
            "observability",
            "grafana",
            "opentelemetry",
            "loki",
            "alertmanager",
            "kafka",
            "vault",
            "flagger",
            "consul",
            "storage",
        ];
        let kind_lang_dir = if flat_kinds.contains(&ctx.kind.as_str()) {
            self.template_dir.join(&ctx.kind)
        } else {
            self.template_dir.join(&ctx.kind).join(&ctx.language)
        };
        if !kind_lang_dir.exists() {
            anyhow::bail!(
                "テンプレートディレクトリが見つかりません: {}",
                kind_lang_dir.display()
            );
        }

        // テンプレートファイルを収集
        let template_files = self.collect_template_files(&kind_lang_dir, ctx)?;

        // 各テンプレートを登録・レンダリング・書き込み
        let mut generated_files = Vec::new();

        for file_info in &template_files {
            let full_template_path = kind_lang_dir.join(&file_info.relative_path);

            // テンプレートファイルの内容を読み込み
            let template_content = fs::read_to_string(&full_template_path)
                .with_context(|| {
                    format!(
                        "テンプレートファイルの読み込みに失敗: {}",
                        full_template_path.display()
                    )
                })?;

            // Tera にテンプレートを登録
            self.tera
                .add_raw_template(&file_info.template_name, &template_content)
                .with_context(|| {
                    format!(
                        "テンプレートの登録に失敗: {}",
                        file_info.template_name
                    )
                })?;

            // レンダリング
            let rendered = self
                .tera
                .render(&file_info.template_name, &tera_ctx)
                .with_context(|| {
                    format!(
                        "テンプレートのレンダリングに失敗: {}",
                        file_info.template_name
                    )
                })?;

            // 出力先のファイルパスを計算
            // .tera 拡張子を除去し、プレースホルダを置換
            let output_relative = Self::resolve_output_path(
                &file_info.relative_path,
                &ctx.service_name,
                &ctx.service_name_snake,
            );
            let output_path = output_dir.join(&output_relative);

            // 親ディレクトリを作成
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "出力ディレクトリの作成に失敗: {}",
                        parent.display()
                    )
                })?;
            }

            // ファイルに書き込み
            fs::write(&output_path, rendered).with_context(|| {
                format!("ファイルの書き込みに失敗: {}", output_path.display())
            })?;

            generated_files.push(output_path);
        }

        Ok(generated_files)
    }

    /// テンプレートディレクトリから条件に合致するファイルを収集する。
    fn collect_template_files(
        &self,
        kind_lang_dir: &Path,
        ctx: &TemplateContext,
    ) -> Result<Vec<TemplateFileInfo>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(kind_lang_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // ディレクトリはスキップ
            if path.is_dir() {
                continue;
            }

            // .tera 拡張子のファイルのみ対象
            let extension = path.extension().and_then(|e| e.to_str());
            if extension != Some("tera") {
                continue;
            }

            // テンプレートディレクトリからの相対パス
            let relative = path
                .strip_prefix(kind_lang_dir)
                .with_context(|| "相対パスの計算に失敗")?
                .to_path_buf();

            // 条件付きファイルのフィルタリング
            if !Self::should_include_file(&relative, ctx) {
                continue;
            }

            let template_name = relative.to_string_lossy().replace('\\', "/");

            files.push(TemplateFileInfo {
                relative_path: relative,
                template_name,
            });
        }

        Ok(files)
    }

    /// ファイルが条件に合致するかを判定する。
    ///
    /// API スタイル固有のファイルや、DB/Kafka/Redis 固有のファイルを
    /// コンテキストの設定に基づいてフィルタリングする。
    fn should_include_file(relative_path: &Path, ctx: &TemplateContext) -> bool {
        let path_str = relative_path.to_string_lossy().replace('\\', "/");

        // API スタイル固有のハンドラファイル
        if path_str.contains("rest_handler") || path_str.contains("rest.rs") {
            return ctx.api_styles.contains(&"rest".to_string());
        }
        if path_str.contains("grpc_handler") || path_str.contains("grpc.rs") {
            return ctx.api_styles.contains(&"grpc".to_string());
        }
        if path_str.contains("graphql_resolver") || path_str.contains("graphql.rs") {
            return ctx.api_styles.contains(&"graphql".to_string());
        }

        // API 定義ファイル / コード生成設定
        if path_str.contains("openapi") || path_str.contains("oapi-codegen") {
            return ctx.api_styles.contains(&"rest".to_string());
        }
        if path_str.contains("proto/") || path_str.ends_with(".proto.tera") {
            return ctx.api_styles.contains(&"grpc".to_string());
        }

        // GraphQL 定義ファイル
        if path_str.contains("schema.graphql") || path_str.contains("gqlgen.yml") {
            return ctx.api_styles.contains(&"graphql".to_string());
        }

        // gRPC ビルド設定ファイル
        if path_str.contains("buf.yaml") || path_str.contains("buf.gen") || path_str.contains("build.rs") {
            return ctx.api_styles.contains(&"grpc".to_string());
        }

        // DB 固有ファイル
        if path_str.contains("persistence") || path_str.contains("db.go") {
            return ctx.has_database;
        }

        // Kafka 固有ファイル
        if path_str.contains("messaging") || path_str.contains("kafka") {
            return ctx.has_kafka;
        }

        // Redis 固有ファイル (D-15)
        if path_str.contains("redis") {
            return ctx.has_redis;
        }

        // それ以外は常に含める
        true
    }

    /// 出力ファイルパスを計算する。
    ///
    /// - .tera 拡張子を除去
    /// - {name} プレースホルダを service_name で置換
    /// - {module} プレースホルダを service_name_snake で置換
    fn resolve_output_path(
        template_relative: &Path,
        service_name: &str,
        service_name_snake: &str,
    ) -> PathBuf {
        let path_str = template_relative.to_string_lossy().replace('\\', "/");

        // .tera 拡張子を除去
        let without_tera = if path_str.ends_with(".tera") {
            &path_str[..path_str.len() - 5]
        } else {
            &path_str
        };

        // プレースホルダの置換
        let resolved = without_tera
            .replace("{name}", service_name)
            .replace("{module}", service_name_snake);

        PathBuf::from(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::context::TemplateContextBuilder;
    use std::fs;
    use tempfile::TempDir;

    // =========================================================================
    // resolve_output_path のテスト
    // =========================================================================

    #[test]
    fn test_resolve_output_path_removes_tera_extension() {
        let result = TemplateEngine::resolve_output_path(
            Path::new("cmd/main.go.tera"),
            "order",
            "order",
        );
        assert_eq!(result, PathBuf::from("cmd/main.go"));
    }

    #[test]
    fn test_resolve_output_path_replaces_name_placeholder() {
        let result = TemplateEngine::resolve_output_path(
            Path::new("{name}.go.tera"),
            "order-api",
            "order_api",
        );
        assert_eq!(result, PathBuf::from("order-api.go"));
    }

    #[test]
    fn test_resolve_output_path_replaces_module_placeholder() {
        let result = TemplateEngine::resolve_output_path(
            Path::new("src/{module}.rs.tera"),
            "order-api",
            "order_api",
        );
        assert_eq!(result, PathBuf::from("src/order_api.rs"));
    }

    #[test]
    fn test_resolve_output_path_nested_with_placeholders() {
        let result = TemplateEngine::resolve_output_path(
            Path::new("lib/{name}.dart.tera"),
            "my-lib",
            "my_lib",
        );
        assert_eq!(result, PathBuf::from("lib/my-lib.dart"));
    }

    // =========================================================================
    // should_include_file のテスト
    // =========================================================================

    #[test]
    fn test_should_include_rest_handler_when_rest() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/rest_handler.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_rest_handler_when_grpc() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("grpc")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/rest_handler.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_grpc_handler_when_grpc() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("grpc")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/grpc_handler.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_grpc_handler_when_rest() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/grpc_handler.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_graphql_resolver_when_graphql() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("graphql")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/graphql_resolver.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_graphql_resolver_when_rest() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/graphql_resolver.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_openapi_when_rest() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("api/openapi/openapi.yaml.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_openapi_when_grpc() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("grpc")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("api/openapi/openapi.yaml.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_proto_when_grpc() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("grpc")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("api/proto/service.proto.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_proto_when_rest() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("api/proto/service.proto.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_persistence_when_database() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .with_database("postgresql")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("internal/infra/persistence/db.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_persistence_when_no_database() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("internal/infra/persistence/db.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_kafka_when_enabled() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .with_kafka()
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("internal/infra/messaging/kafka.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_kafka_when_disabled() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("internal/infra/messaging/kafka.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_common_files_always() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        // 共通ファイルは常に含まれる
        assert!(TemplateEngine::should_include_file(
            Path::new("cmd/main.go.tera"),
            &ctx,
        ));
        assert!(TemplateEngine::should_include_file(
            Path::new("go.mod.tera"),
            &ctx,
        ));
        assert!(TemplateEngine::should_include_file(
            Path::new("Dockerfile.tera"),
            &ctx,
        ));
        assert!(TemplateEngine::should_include_file(
            Path::new("config/config.yaml.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_oapi_codegen_when_rest() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("oapi-codegen.yaml.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_oapi_codegen_when_grpc() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("grpc")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("oapi-codegen.yaml.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_include_readme_always() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("README.md.tera"),
            &ctx,
        ));
    }

    // =========================================================================
    // Rust サーバーの should_include_file テスト
    // =========================================================================

    #[test]
    fn test_should_include_rust_rest_handler_when_rest() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .api_style("rest")
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("src/adapter/handler/rest.rs.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_rust_grpc_handler_when_rest() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .api_style("rest")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("src/adapter/handler/grpc.rs.tera"),
            &ctx,
        ));
    }

    // =========================================================================
    // D-15: has_redis filter のテスト
    // =========================================================================

    #[test]
    fn test_should_include_redis_file_when_redis_enabled() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .with_redis()
            .build();

        assert!(TemplateEngine::should_include_file(
            Path::new("internal/infra/cache/redis.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_redis_file_when_redis_disabled() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("internal/infra/cache/redis.go.tera"),
            &ctx,
        ));
    }

    #[test]
    fn test_should_exclude_redis_config_when_redis_disabled() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .api_style("rest")
            .build();

        assert!(!TemplateEngine::should_include_file(
            Path::new("src/infra/redis_client.rs.tera"),
            &ctx,
        ));
    }

    // =========================================================================
    // D-04: api_styles Vec 対応のテスト
    // =========================================================================

    #[test]
    fn test_should_include_rest_and_grpc_when_both_selected() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_styles(vec!["rest".to_string(), "grpc".to_string()])
            .build();

        // REST ハンドラ: 含まれる
        assert!(TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/rest_handler.go.tera"),
            &ctx,
        ));
        // gRPC ハンドラ: 含まれる
        assert!(TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/grpc_handler.go.tera"),
            &ctx,
        ));
        // GraphQL: 含まれない
        assert!(!TemplateEngine::should_include_file(
            Path::new("internal/adapter/handler/graphql_resolver.go.tera"),
            &ctx,
        ));
        // OpenAPI: 含まれる (REST)
        assert!(TemplateEngine::should_include_file(
            Path::new("api/openapi/openapi.yaml.tera"),
            &ctx,
        ));
        // Proto: 含まれる (gRPC)
        assert!(TemplateEngine::should_include_file(
            Path::new("api/proto/service.proto.tera"),
            &ctx,
        ));
    }

    // =========================================================================
    // TemplateEngine::new のテスト
    // =========================================================================

    #[test]
    fn test_engine_new_with_valid_dir() {
        let tmp = TempDir::new().unwrap();
        let engine = TemplateEngine::new(tmp.path());
        assert!(engine.is_ok());
    }

    // =========================================================================
    // render_to_dir の統合テスト
    // =========================================================================

    #[test]
    fn test_render_go_server_rest() {
        // テスト用のテンプレートディレクトリを作成
        let tmp = TempDir::new().unwrap();
        let template_root = tmp.path().join("templates");
        let go_server_dir = template_root.join("server").join("go");

        // テンプレートファイルを配置
        // cmd/main.go.tera
        let cmd_dir = go_server_dir.join("cmd");
        fs::create_dir_all(&cmd_dir).unwrap();
        fs::write(
            cmd_dir.join("main.go.tera"),
            "package main\n\n// Service: {{ service_name }}\n// Module: {{ go_module }}\n\nfunc main() {}\n",
        )
        .unwrap();

        // go.mod.tera
        fs::write(
            go_server_dir.join("go.mod.tera"),
            "module {{ go_module }}\n\ngo 1.23\n",
        )
        .unwrap();

        // Dockerfile.tera
        fs::write(
            go_server_dir.join("Dockerfile.tera"),
            "FROM golang:1.23\nWORKDIR /app\n",
        )
        .unwrap();

        // config/config.yaml.tera
        let config_dir = go_server_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.yaml.tera"),
            "service_name: {{ service_name }}\ntier: {{ tier }}\n",
        )
        .unwrap();

        // internal/adapter/handler/rest_handler.go.tera (REST 固有)
        let handler_dir = go_server_dir
            .join("internal")
            .join("adapter")
            .join("handler");
        fs::create_dir_all(&handler_dir).unwrap();
        fs::write(
            handler_dir.join("rest_handler.go.tera"),
            "package handler\n\n// REST handler for {{ service_name_pascal }}\n",
        )
        .unwrap();

        // internal/adapter/handler/grpc_handler.go.tera (gRPC 固有 -> REST では除外)
        fs::write(
            handler_dir.join("grpc_handler.go.tera"),
            "package handler\n\n// gRPC handler\n",
        )
        .unwrap();

        // internal/domain/model/entity.go.tera (共通)
        let model_dir = go_server_dir.join("internal").join("domain").join("model");
        fs::create_dir_all(&model_dir).unwrap();
        fs::write(
            model_dir.join("entity.go.tera"),
            "package model\n\ntype {{ service_name_pascal }} struct {}\n",
        )
        .unwrap();

        // internal/usecase/usecase.go.tera (共通)
        let usecase_dir = go_server_dir.join("internal").join("usecase");
        fs::create_dir_all(&usecase_dir).unwrap();
        fs::write(
            usecase_dir.join("usecase.go.tera"),
            "package usecase\n\ntype {{ service_name_pascal }}UseCase struct {}\n",
        )
        .unwrap();

        // api/openapi/openapi.yaml.tera (REST 固有)
        let openapi_dir = go_server_dir.join("api").join("openapi");
        fs::create_dir_all(&openapi_dir).unwrap();
        fs::write(
            openapi_dir.join("openapi.yaml.tera"),
            "openapi: \"3.0.0\"\ninfo:\n  title: {{ service_name_pascal }} API\n",
        )
        .unwrap();

        // api/proto/service.proto.tera (gRPC 固有 -> REST では除外)
        let proto_dir = go_server_dir.join("api").join("proto");
        fs::create_dir_all(&proto_dir).unwrap();
        fs::write(
            proto_dir.join("service.proto.tera"),
            "syntax = \"proto3\";\npackage {{ service_name_snake }};\n",
        )
        .unwrap();

        // 出力先ディレクトリ
        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        // コンテキストを構築
        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .api_style("rest")
            .build();

        // テンプレートエンジンでレンダリング
        let mut engine = TemplateEngine::new(&template_root).unwrap();
        let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

        // 生成されたファイルを検証
        // REST のため rest_handler は含まれ、grpc_handler と proto は除外される
        let generated_names: Vec<String> = generated
            .iter()
            .map(|p| {
                p.strip_prefix(&output_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();

        assert!(
            generated_names.contains(&"cmd/main.go".to_string()),
            "cmd/main.go should be generated"
        );
        assert!(
            generated_names.contains(&"go.mod".to_string()),
            "go.mod should be generated"
        );
        assert!(
            generated_names.contains(&"Dockerfile".to_string()),
            "Dockerfile should be generated"
        );
        assert!(
            generated_names.contains(&"config/config.yaml".to_string()),
            "config/config.yaml should be generated"
        );
        assert!(
            generated_names.contains(&"internal/adapter/handler/rest_handler.go".to_string()),
            "REST handler should be generated"
        );
        assert!(
            !generated_names.contains(&"internal/adapter/handler/grpc_handler.go".to_string()),
            "gRPC handler should NOT be generated for REST"
        );
        assert!(
            generated_names.contains(&"api/openapi/openapi.yaml".to_string()),
            "OpenAPI spec should be generated for REST"
        );
        assert!(
            !generated_names.contains(&"api/proto/service.proto".to_string()),
            "Proto file should NOT be generated for REST"
        );
        assert!(
            generated_names.contains(&"internal/domain/model/entity.go".to_string()),
            "Domain model should be generated"
        );
        assert!(
            generated_names.contains(&"internal/usecase/usecase.go".to_string()),
            "UseCase should be generated"
        );

        // レンダリング内容の検証
        let main_content = fs::read_to_string(output_dir.join("cmd/main.go")).unwrap();
        assert!(main_content.contains("// Service: order-api"));
        assert!(main_content.contains(
            "// Module: github.com/org/k1s0/regions/service/order-api/server/go"
        ));

        let go_mod_content = fs::read_to_string(output_dir.join("go.mod")).unwrap();
        assert!(go_mod_content
            .contains("module github.com/org/k1s0/regions/service/order-api/server/go"));

        let entity_content =
            fs::read_to_string(output_dir.join("internal/domain/model/entity.go")).unwrap();
        assert!(entity_content.contains("type OrderApi struct {}"));

        let config_content = fs::read_to_string(output_dir.join("config/config.yaml")).unwrap();
        assert!(config_content.contains("service_name: order-api"));
        assert!(config_content.contains("tier: service"));
    }

    #[test]
    fn test_render_go_server_grpc() {
        // gRPC サーバーのテンプレートレンダリングをテスト
        let tmp = TempDir::new().unwrap();
        let template_root = tmp.path().join("templates");
        let go_server_dir = template_root.join("server").join("go");

        // テンプレートファイルを配置
        let handler_dir = go_server_dir
            .join("internal")
            .join("adapter")
            .join("handler");
        fs::create_dir_all(&handler_dir).unwrap();

        fs::write(
            handler_dir.join("rest_handler.go.tera"),
            "package handler\n// REST\n",
        )
        .unwrap();
        fs::write(
            handler_dir.join("grpc_handler.go.tera"),
            "package handler\n// gRPC for {{ service_name_pascal }}\n",
        )
        .unwrap();

        let proto_dir = go_server_dir.join("api").join("proto");
        fs::create_dir_all(&proto_dir).unwrap();
        fs::write(
            proto_dir.join("service.proto.tera"),
            "syntax = \"proto3\";\npackage {{ service_name_snake }};\n",
        )
        .unwrap();

        let openapi_dir = go_server_dir.join("api").join("openapi");
        fs::create_dir_all(&openapi_dir).unwrap();
        fs::write(
            openapi_dir.join("openapi.yaml.tera"),
            "openapi: 3.0\n",
        )
        .unwrap();

        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .api_style("grpc")
            .build();

        let mut engine = TemplateEngine::new(&template_root).unwrap();
        let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

        let generated_names: Vec<String> = generated
            .iter()
            .map(|p| {
                p.strip_prefix(&output_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();

        // gRPC のため grpc_handler と proto は含まれ、rest_handler と openapi は除外
        assert!(generated_names.contains(&"internal/adapter/handler/grpc_handler.go".to_string()));
        assert!(generated_names.contains(&"api/proto/service.proto".to_string()));
        assert!(!generated_names.contains(&"internal/adapter/handler/rest_handler.go".to_string()));
        assert!(!generated_names.contains(&"api/openapi/openapi.yaml".to_string()));

        // gRPC ハンドラの内容検証
        let grpc_content = fs::read_to_string(
            output_dir.join("internal/adapter/handler/grpc_handler.go"),
        )
        .unwrap();
        assert!(grpc_content.contains("// gRPC for OrderApi"));
    }

    #[test]
    fn test_render_with_database_and_kafka() {
        let tmp = TempDir::new().unwrap();
        let template_root = tmp.path().join("templates");
        let go_server_dir = template_root.join("server").join("go");

        // DB 固有テンプレート
        let persistence_dir = go_server_dir.join("internal").join("infra").join("persistence");
        fs::create_dir_all(&persistence_dir).unwrap();
        fs::write(
            persistence_dir.join("db.go.tera"),
            "package persistence\n\n// DB type: {{ database_type }}\n",
        )
        .unwrap();

        // Kafka 固有テンプレート
        let messaging_dir = go_server_dir.join("internal").join("infra").join("messaging");
        fs::create_dir_all(&messaging_dir).unwrap();
        fs::write(
            messaging_dir.join("kafka.go.tera"),
            "package messaging\n\n// Kafka producer\n",
        )
        .unwrap();

        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        // DB あり、Kafka あり
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .with_database("postgresql")
            .with_kafka()
            .build();

        let mut engine = TemplateEngine::new(&template_root).unwrap();
        let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

        let generated_names: Vec<String> = generated
            .iter()
            .map(|p| {
                p.strip_prefix(&output_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();

        assert!(generated_names.contains(&"internal/infra/persistence/db.go".to_string()));
        assert!(generated_names.contains(&"internal/infra/messaging/kafka.go".to_string()));

        // DB 内容の検証
        let db_content =
            fs::read_to_string(output_dir.join("internal/infra/persistence/db.go")).unwrap();
        assert!(db_content.contains("// DB type: postgresql"));
    }

    #[test]
    fn test_render_without_database_and_kafka() {
        let tmp = TempDir::new().unwrap();
        let template_root = tmp.path().join("templates");
        let go_server_dir = template_root.join("server").join("go");

        // DB 固有テンプレート
        let persistence_dir = go_server_dir.join("internal").join("infra").join("persistence");
        fs::create_dir_all(&persistence_dir).unwrap();
        fs::write(
            persistence_dir.join("db.go.tera"),
            "package persistence\n",
        )
        .unwrap();

        // Kafka 固有テンプレート
        let messaging_dir = go_server_dir.join("internal").join("infra").join("messaging");
        fs::create_dir_all(&messaging_dir).unwrap();
        fs::write(
            messaging_dir.join("kafka.go.tera"),
            "package messaging\n",
        )
        .unwrap();

        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        // DB なし、Kafka なし
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        let mut engine = TemplateEngine::new(&template_root).unwrap();
        let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

        let generated_names: Vec<String> = generated
            .iter()
            .map(|p| {
                p.strip_prefix(&output_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();

        assert!(!generated_names.contains(&"internal/infra/persistence/db.go".to_string()));
        assert!(!generated_names.contains(&"internal/infra/messaging/kafka.go".to_string()));
    }

    #[test]
    fn test_render_library_with_placeholders() {
        // ライブラリテンプレートのプレースホルダ置換をテスト
        let tmp = TempDir::new().unwrap();
        let template_root = tmp.path().join("templates");
        let go_lib_dir = template_root.join("library").join("go");

        fs::create_dir_all(&go_lib_dir).unwrap();

        // {name}.go.tera -> order-api.go
        fs::write(
            go_lib_dir.join("{name}.go.tera"),
            "package {{ service_name_snake }}\n\n// {{ service_name_pascal }} library\n",
        )
        .unwrap();

        // go.mod.tera
        fs::write(
            go_lib_dir.join("go.mod.tera"),
            "module {{ go_module }}\n\ngo 1.23\n",
        )
        .unwrap();

        // internal/internal.go.tera
        let internal_dir = go_lib_dir.join("internal");
        fs::create_dir_all(&internal_dir).unwrap();
        fs::write(
            internal_dir.join("internal.go.tera"),
            "package internal\n",
        )
        .unwrap();

        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        let ctx = TemplateContextBuilder::new("order-api", "system", "go", "library")
            .build();

        let mut engine = TemplateEngine::new(&template_root).unwrap();
        let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

        let generated_names: Vec<String> = generated
            .iter()
            .map(|p| {
                p.strip_prefix(&output_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();

        // {name} が "order-api" に置換されていることを検証
        assert!(
            generated_names.contains(&"order-api.go".to_string()),
            "Placeholder {{name}} should be replaced with service_name. Generated: {:?}",
            generated_names
        );
        assert!(generated_names.contains(&"go.mod".to_string()));
        assert!(generated_names.contains(&"internal/internal.go".to_string()));

        // 内容の検証
        let lib_content = fs::read_to_string(output_dir.join("order-api.go")).unwrap();
        assert!(lib_content.contains("package order_api"));
        assert!(lib_content.contains("// OrderApi library"));
    }

    #[test]
    fn test_render_rust_library_with_module_placeholder() {
        // Rust ライブラリの {module} プレースホルダ置換をテスト
        let tmp = TempDir::new().unwrap();
        let template_root = tmp.path().join("templates");
        let rust_lib_dir = template_root.join("library").join("rust");
        let src_dir = rust_lib_dir.join("src");

        fs::create_dir_all(&src_dir).unwrap();

        // src/lib.rs.tera
        fs::write(
            src_dir.join("lib.rs.tera"),
            "pub mod {{ service_name_snake }};\n",
        )
        .unwrap();

        // src/{module}.rs.tera -> src/order_api.rs
        fs::write(
            src_dir.join("{module}.rs.tera"),
            "// Module: {{ service_name_pascal }}\n",
        )
        .unwrap();

        // Cargo.toml.tera
        fs::write(
            rust_lib_dir.join("Cargo.toml.tera"),
            "[package]\nname = \"{{ rust_crate }}\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        let ctx = TemplateContextBuilder::new("order-api", "service", "rust", "library")
            .build();

        let mut engine = TemplateEngine::new(&template_root).unwrap();
        let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

        let generated_names: Vec<String> = generated
            .iter()
            .map(|p| {
                p.strip_prefix(&output_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();

        // {module} が "order_api" (snake_case) に置換されていることを検証
        assert!(
            generated_names.contains(&"src/order_api.rs".to_string()),
            "Placeholder {{module}} should be replaced with service_name_snake. Generated: {:?}",
            generated_names
        );
        assert!(generated_names.contains(&"src/lib.rs".to_string()));
        assert!(generated_names.contains(&"Cargo.toml".to_string()));

        // 内容の検証
        let lib_rs = fs::read_to_string(output_dir.join("src/lib.rs")).unwrap();
        assert!(lib_rs.contains("pub mod order_api;"));

        let module_rs = fs::read_to_string(output_dir.join("src/order_api.rs")).unwrap();
        assert!(module_rs.contains("// Module: OrderApi"));

        let cargo_toml = fs::read_to_string(output_dir.join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains("name = \"order-api\""));
    }

    #[test]
    fn test_render_nonexistent_template_dir_returns_error() {
        let tmp = TempDir::new().unwrap();
        let template_root = tmp.path().join("templates");
        // テンプレートディレクトリを作成しない

        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        let mut engine = TemplateEngine::new(&template_root).unwrap();
        let result = engine.render_to_dir(&ctx, &output_dir);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("テンプレートディレクトリが見つかりません"),
            "Expected directory not found error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_render_with_actual_templates() {
        // 実際のプロジェクトのテンプレートディレクトリを使用した統合テスト
        // CI 環境ではテンプレートディレクトリが存在しない場合があるため、
        // 存在チェックを行う
        let template_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        if !template_dir.join("server").join("go").exists() {
            // テンプレートが存在しない場合はスキップ
            return;
        }

        let tmp = TempDir::new().unwrap();
        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .api_style("rest")
            .with_database("postgresql")
            .with_redis()
            .build();

        let mut engine = TemplateEngine::new(&template_dir).unwrap();
        let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

        // ファイルが生成されたことを検証
        assert!(!generated.is_empty(), "No files were generated");

        // REST のため openapi は含まれ、proto は含まれない
        let generated_names: Vec<String> = generated
            .iter()
            .map(|p| {
                p.strip_prefix(&output_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();

        // 基本ファイルの存在確認
        assert!(
            generated_names.iter().any(|n| n.contains("main.go")),
            "main.go should be generated. Files: {:?}",
            generated_names
        );
        assert!(
            generated_names.iter().any(|n| n.contains("go.mod")),
            "go.mod should be generated. Files: {:?}",
            generated_names
        );
    }
}
