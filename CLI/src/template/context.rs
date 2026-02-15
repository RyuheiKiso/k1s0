use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use serde::Serialize;
use tera::Context;

/// Docker レジストリのデフォルト値。
const DEFAULT_DOCKER_REGISTRY: &str = "harbor.internal.example.com";

/// テンプレートに渡すプロジェクト情報（旧 API との互換性維持）。
#[derive(Debug, Clone, Serialize)]
pub struct ProjectContext {
    /// プロジェクト名 (kebab-case)
    pub name: String,
    /// リージョン (system / business / service)
    pub region: String,
    /// プロジェクト種別 (go-server / rust-server / react-client / flutter-client)
    pub project_type: String,
}

/// ProjectContext から Tera の Context を構築する（旧 API との互換性維持）。
pub fn build_context(project: &ProjectContext) -> Context {
    let mut ctx = Context::new();
    ctx.insert("project", project);
    ctx
}

/// テンプレートエンジンに渡す全変数を保持する構造体。
///
/// CLI の対話フローで収集した入力値から、テンプレートエンジン仕様に定義された
/// 全変数を自動導出して保持する。
#[derive(Debug, Clone, Serialize)]
pub struct TemplateContext {
    /// サービス名 (kebab-case, 正規形)
    pub service_name: String,
    /// サービス名 (snake_case, 自動導出)
    pub service_name_snake: String,
    /// サービス名 (PascalCase, 自動導出)
    pub service_name_pascal: String,
    /// サービス名 (camelCase, 自動導出)
    pub service_name_camel: String,
    /// 配置階層: system / business / service
    pub tier: String,
    /// regions/ からの相対パス (自動導出)
    pub module_path: String,
    /// 言語識別子: go / rust / typescript / dart
    pub language: String,
    /// 種別: server / client / library / database
    pub kind: String,
    /// API 方式 (server 時, 後方互換用: api_styles の最初の要素): rest / grpc / graphql
    pub api_style: String,
    /// API 方式一覧 (server 時, 複数選択対応): vec!["rest", "grpc", ...]
    pub api_styles: Vec<String>,
    /// DB 有無
    pub has_database: bool,
    /// RDBMS 種別 (DB 有効時): postgresql / mysql / sqlite
    pub database_type: String,
    /// Kafka 有無
    pub has_kafka: bool,
    /// Redis 有無
    pub has_redis: bool,
    /// Go モジュールパス (Go 時, 自動導出)
    pub go_module: String,
    /// Rust クレート名 (Rust 時, 自動導出)
    pub rust_crate: String,
    /// Docker レジストリ
    pub docker_registry: String,
    /// Docker プロジェクト名 (自動導出: "k1s0-{tier}")
    pub docker_project: String,
}

/// TemplateContext を構築するためのビルダー。
///
/// CLI の対話フローで収集した最小限の入力値から、
/// テンプレートエンジン仕様の導出ルールに従って全変数を自動計算する。
#[derive(Debug, Clone)]
pub struct TemplateContextBuilder {
    service_name: String,
    tier: String,
    language: String,
    kind: String,
    api_styles: Vec<String>,
    has_database: bool,
    database_type: String,
    has_kafka: bool,
    has_redis: bool,
    docker_registry: String,
    go_module_base: String,
}

impl TemplateContextBuilder {
    /// 必須パラメータを指定してビルダーを作成する。
    ///
    /// # Arguments
    /// * `service_name` - サービス名 (kebab-case)
    /// * `tier` - 配置階層 (system / business / service)
    /// * `language` - 言語識別子 (go / rust / typescript / dart)
    /// * `kind` - 種別 (server / client / library / database)
    pub fn new(service_name: &str, tier: &str, language: &str, kind: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            tier: tier.to_string(),
            language: language.to_string(),
            kind: kind.to_string(),
            api_styles: Vec::new(),
            has_database: false,
            database_type: String::new(),
            has_kafka: false,
            has_redis: false,
            docker_registry: DEFAULT_DOCKER_REGISTRY.to_string(),
            go_module_base: "github.com/org/k1s0".to_string(),
        }
    }

    /// API 方式を設定する (server 時)。単一スタイルの後方互換 API。
    pub fn api_style(mut self, api_style: &str) -> Self {
        if self.api_styles.is_empty() {
            self.api_styles = vec![api_style.to_string()];
        } else {
            self.api_styles = vec![api_style.to_string()];
        }
        self
    }

    /// 複数の API 方式を設定する (server 時)。
    pub fn api_styles(mut self, styles: Vec<String>) -> Self {
        self.api_styles = styles;
        self
    }

    /// データベース設定を有効にする。
    pub fn with_database(mut self, database_type: &str) -> Self {
        self.has_database = true;
        self.database_type = database_type.to_string();
        self
    }

    /// Kafka を有効にする。
    pub fn with_kafka(mut self) -> Self {
        self.has_kafka = true;
        self
    }

    /// Redis を有効にする。
    pub fn with_redis(mut self) -> Self {
        self.has_redis = true;
        self
    }

    /// Docker レジストリを設定する。
    pub fn docker_registry(mut self, registry: &str) -> Self {
        self.docker_registry = registry.to_string();
        self
    }

    /// Go モジュールベースパスを設定する。
    pub fn go_module_base(mut self, base: &str) -> Self {
        self.go_module_base = base.to_string();
        self
    }

    /// TemplateContext を構築する。
    ///
    /// 入力値から導出ルールに従って全変数を自動計算する。
    pub fn build(self) -> TemplateContext {
        // ケース変換の導出
        let service_name_snake = self.service_name.to_snake_case();
        let service_name_pascal = self.service_name.to_pascal_case();
        let service_name_camel = self.service_name.to_lower_camel_case();

        // module_path の導出:
        // "regions/{tier}/{service_name}/{kind}/{language}"
        let module_path = format!(
            "regions/{}/{}/{}/{}",
            self.tier, self.service_name, self.kind, self.language
        );

        // go_module の導出:
        // "{go_module_base}/{module_path}"
        let go_module = if self.language == "go" {
            format!("{}/{}", self.go_module_base, module_path)
        } else {
            String::new()
        };

        // rust_crate の導出: service_name をそのまま使用
        let rust_crate = if self.language == "rust" {
            self.service_name.clone()
        } else {
            String::new()
        };

        // docker_project の導出: "k1s0-{tier}"
        let docker_project = format!("k1s0-{}", self.tier);

        // api_style: 後方互換のため api_styles の先頭要素を設定
        let api_style = self.api_styles.first().cloned().unwrap_or_default();

        TemplateContext {
            service_name: self.service_name,
            service_name_snake,
            service_name_pascal,
            service_name_camel,
            tier: self.tier,
            module_path,
            language: self.language,
            kind: self.kind,
            api_style,
            api_styles: self.api_styles,
            has_database: self.has_database,
            database_type: self.database_type,
            has_kafka: self.has_kafka,
            has_redis: self.has_redis,
            go_module,
            rust_crate,
            docker_registry: self.docker_registry,
            docker_project,
        }
    }
}

impl TemplateContext {
    /// TemplateContext を Tera の Context に変換する。
    ///
    /// 全フィールドを個別の変数として Context に挿入する。
    /// テンプレート内で `{{ service_name }}` のようにフラットにアクセスできる。
    pub fn to_tera_context(&self) -> Context {
        let mut ctx = Context::new();
        ctx.insert("service_name", &self.service_name);
        ctx.insert("service_name_snake", &self.service_name_snake);
        ctx.insert("service_name_pascal", &self.service_name_pascal);
        ctx.insert("service_name_camel", &self.service_name_camel);
        ctx.insert("tier", &self.tier);
        ctx.insert("module_path", &self.module_path);
        ctx.insert("language", &self.language);
        ctx.insert("kind", &self.kind);
        ctx.insert("api_style", &self.api_style);
        ctx.insert("api_styles", &self.api_styles);
        ctx.insert("has_database", &self.has_database);
        ctx.insert("database_type", &self.database_type);
        ctx.insert("has_kafka", &self.has_kafka);
        ctx.insert("has_redis", &self.has_redis);
        ctx.insert("go_module", &self.go_module);
        ctx.insert("rust_crate", &self.rust_crate);
        ctx.insert("docker_registry", &self.docker_registry);
        ctx.insert("docker_project", &self.docker_project);
        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 旧 API (ProjectContext) の互換性テスト
    // =========================================================================

    #[test]
    fn test_build_context_legacy() {
        let project = ProjectContext {
            name: "my-service".to_string(),
            region: "service".to_string(),
            project_type: "go-server".to_string(),
        };
        let ctx = build_context(&project);
        let json = ctx.into_json();
        assert!(json.get("project").is_some());
    }

    #[test]
    fn test_project_context_serialize() {
        let project = ProjectContext {
            name: "test-project".to_string(),
            region: "system".to_string(),
            project_type: "rust-server".to_string(),
        };
        let json = serde_json::to_value(&project).unwrap();
        assert_eq!(json["name"], "test-project");
        assert_eq!(json["region"], "system");
        assert_eq!(json["project_type"], "rust-server");
    }

    // =========================================================================
    // service_name からのケース変換テスト
    // =========================================================================

    #[test]
    fn test_context_name_derivation_order_api() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .build();

        assert_eq!(ctx.service_name, "order-api");
        assert_eq!(ctx.service_name_snake, "order_api");
        assert_eq!(ctx.service_name_pascal, "OrderApi");
        assert_eq!(ctx.service_name_camel, "orderApi");
    }

    #[test]
    fn test_context_name_derivation_user_auth_service() {
        let ctx = TemplateContextBuilder::new("user-auth-service", "service", "go", "server")
            .build();

        assert_eq!(ctx.service_name, "user-auth-service");
        assert_eq!(ctx.service_name_snake, "user_auth_service");
        assert_eq!(ctx.service_name_pascal, "UserAuthService");
        assert_eq!(ctx.service_name_camel, "userAuthService");
    }

    #[test]
    fn test_context_name_derivation_single_word() {
        let ctx = TemplateContextBuilder::new("inventory", "service", "go", "server")
            .build();

        assert_eq!(ctx.service_name, "inventory");
        assert_eq!(ctx.service_name_snake, "inventory");
        assert_eq!(ctx.service_name_pascal, "Inventory");
        assert_eq!(ctx.service_name_camel, "inventory");
    }

    // =========================================================================
    // module_path の導出テスト
    // =========================================================================

    #[test]
    fn test_context_module_path_service_go_server() {
        // tier=service, service_name=order, kind=server, lang=go
        // -> "regions/service/order/server/go"
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert_eq!(ctx.module_path, "regions/service/order/server/go");
    }

    #[test]
    fn test_context_module_path_business_client_react() {
        // tier=business, service_name=crm, kind=client, lang=react
        // -> "regions/business/crm/client/react"
        let ctx = TemplateContextBuilder::new("crm", "business", "react", "client")
            .build();

        assert_eq!(ctx.module_path, "regions/business/crm/client/react");
    }

    #[test]
    fn test_context_module_path_system_library_rust() {
        // tier=system, service_name=auth, kind=library, lang=rust
        // -> "regions/system/auth/library/rust"
        let ctx = TemplateContextBuilder::new("auth", "system", "rust", "library")
            .build();

        assert_eq!(ctx.module_path, "regions/system/auth/library/rust");
    }

    // =========================================================================
    // docker_project の導出テスト
    // =========================================================================

    #[test]
    fn test_context_docker_project_system() {
        let ctx = TemplateContextBuilder::new("auth", "system", "go", "server")
            .build();

        assert_eq!(ctx.docker_project, "k1s0-system");
    }

    #[test]
    fn test_context_docker_project_business() {
        let ctx = TemplateContextBuilder::new("crm", "business", "go", "server")
            .build();

        assert_eq!(ctx.docker_project, "k1s0-business");
    }

    #[test]
    fn test_context_docker_project_service() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert_eq!(ctx.docker_project, "k1s0-service");
    }

    // =========================================================================
    // go_module の導出テスト
    // =========================================================================

    #[test]
    fn test_go_module_for_go_project() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert_eq!(
            ctx.go_module,
            "github.com/org/k1s0/regions/service/order/server/go"
        );
    }

    #[test]
    fn test_go_module_empty_for_non_go() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .build();

        assert_eq!(ctx.go_module, "");
    }

    // =========================================================================
    // rust_crate の導出テスト
    // =========================================================================

    #[test]
    fn test_rust_crate_for_rust_project() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "rust", "server")
            .build();

        assert_eq!(ctx.rust_crate, "order-api");
    }

    #[test]
    fn test_rust_crate_empty_for_non_rust() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .build();

        assert_eq!(ctx.rust_crate, "");
    }

    // =========================================================================
    // ビルダーのオプション設定テスト
    // =========================================================================

    #[test]
    fn test_builder_api_style() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("rest")
            .build();

        assert_eq!(ctx.api_style, "rest");
    }

    #[test]
    fn test_builder_with_database() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .with_database("postgresql")
            .build();

        assert!(ctx.has_database);
        assert_eq!(ctx.database_type, "postgresql");
    }

    #[test]
    fn test_builder_without_database() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert!(!ctx.has_database);
        assert_eq!(ctx.database_type, "");
    }

    #[test]
    fn test_builder_with_kafka() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .with_kafka()
            .build();

        assert!(ctx.has_kafka);
    }

    #[test]
    fn test_builder_with_redis() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .with_redis()
            .build();

        assert!(ctx.has_redis);
    }

    // --- D-04: api_styles Vec ---

    #[test]
    fn test_builder_api_styles_multiple() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_styles(vec!["rest".to_string(), "grpc".to_string()])
            .build();

        assert_eq!(ctx.api_styles, vec!["rest".to_string(), "grpc".to_string()]);
        assert_eq!(ctx.api_style, "rest"); // backward compat: first element
    }

    #[test]
    fn test_builder_api_style_backward_compat() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .api_style("grpc")
            .build();

        assert_eq!(ctx.api_styles, vec!["grpc".to_string()]);
        assert_eq!(ctx.api_style, "grpc");
    }

    #[test]
    fn test_builder_api_styles_empty_default() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert!(ctx.api_styles.is_empty());
        assert_eq!(ctx.api_style, ""); // no styles = empty string
    }

    // --- D-09: go_module_base ---

    #[test]
    fn test_builder_go_module_base_custom() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .go_module_base("github.com/myorg/myrepo")
            .build();

        assert_eq!(ctx.go_module, "github.com/myorg/myrepo/regions/service/order/server/go");
    }

    #[test]
    fn test_builder_go_module_base_default() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert_eq!(ctx.go_module, "github.com/org/k1s0/regions/service/order/server/go");
    }

    #[test]
    fn test_builder_docker_registry_default() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .build();

        assert_eq!(ctx.docker_registry, "harbor.internal.example.com");
    }

    #[test]
    fn test_builder_docker_registry_custom() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .docker_registry("custom.registry.io")
            .build();

        assert_eq!(ctx.docker_registry, "custom.registry.io");
    }

    #[test]
    fn test_builder_full_options() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .api_style("rest")
            .with_database("postgresql")
            .with_kafka()
            .with_redis()
            .build();

        assert_eq!(ctx.service_name, "order-api");
        assert_eq!(ctx.service_name_snake, "order_api");
        assert_eq!(ctx.service_name_pascal, "OrderApi");
        assert_eq!(ctx.service_name_camel, "orderApi");
        assert_eq!(ctx.tier, "service");
        assert_eq!(ctx.module_path, "regions/service/order-api/server/go");
        assert_eq!(ctx.language, "go");
        assert_eq!(ctx.kind, "server");
        assert_eq!(ctx.api_style, "rest");
        assert!(ctx.has_database);
        assert_eq!(ctx.database_type, "postgresql");
        assert!(ctx.has_kafka);
        assert!(ctx.has_redis);
        assert_eq!(
            ctx.go_module,
            "github.com/org/k1s0/regions/service/order-api/server/go"
        );
        assert_eq!(ctx.rust_crate, "");
        assert_eq!(ctx.docker_registry, "harbor.internal.example.com");
        assert_eq!(ctx.docker_project, "k1s0-service");
    }

    // =========================================================================
    // to_tera_context のテスト
    // =========================================================================

    #[test]
    fn test_to_tera_context_contains_all_fields() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .api_style("rest")
            .with_database("postgresql")
            .with_kafka()
            .with_redis()
            .build();

        let tera_ctx = ctx.to_tera_context();
        let json = tera_ctx.into_json();

        assert_eq!(json["service_name"], "order-api");
        assert_eq!(json["service_name_snake"], "order_api");
        assert_eq!(json["service_name_pascal"], "OrderApi");
        assert_eq!(json["service_name_camel"], "orderApi");
        assert_eq!(json["tier"], "service");
        assert_eq!(json["module_path"], "regions/service/order-api/server/go");
        assert_eq!(json["language"], "go");
        assert_eq!(json["kind"], "server");
        assert_eq!(json["api_style"], "rest");
        assert_eq!(json["api_styles"], serde_json::json!(["rest"]));
        assert_eq!(json["has_database"], true);
        assert_eq!(json["database_type"], "postgresql");
        assert_eq!(json["has_kafka"], true);
        assert_eq!(json["has_redis"], true);
        assert_eq!(
            json["go_module"],
            "github.com/org/k1s0/regions/service/order-api/server/go"
        );
        assert_eq!(json["rust_crate"], "");
        assert_eq!(json["docker_registry"], "harbor.internal.example.com");
        assert_eq!(json["docker_project"], "k1s0-service");
    }

    #[test]
    fn test_to_tera_context_flat_access() {
        // テンプレートで {{ service_name }} のようにフラットアクセスできることを検証
        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .build();

        let tera_ctx = ctx.to_tera_context();
        let json = tera_ctx.into_json();

        // フラットにアクセスできる（project.service_name ではなく service_name）
        assert!(json.get("service_name").is_some());
        assert!(json.get("project").is_none()); // ネストされていない
    }

    // =========================================================================
    // TemplateContext の Serialize テスト
    // =========================================================================

    #[test]
    fn test_template_context_serialize() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
            .api_style("rest")
            .build();

        let json = serde_json::to_value(&ctx).unwrap();
        assert_eq!(json["service_name"], "order-api");
        assert_eq!(json["service_name_snake"], "order_api");
        assert_eq!(json["tier"], "service");
        assert_eq!(json["kind"], "server");
    }
}
