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

/// `ProjectContext` から Tera の Context を構築する（旧 API との互換性維持）。
pub fn build_context(project: &ProjectContext) -> Context {
    let mut ctx = Context::new();
    ctx.insert("project", project);
    ctx
}

/// テンプレートエンジンに渡す全変数を保持する構造体。
///
/// CLI の対話フローで収集した入力値から、テンプレートエンジン仕様に定義された
/// 全変数を自動導出して保持する。
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize)]
pub struct TemplateContext {
    /// サービス名 (kebab-case, 正規形)
    pub service_name: String,
    /// サービス名 (`snake_case`, 自動導出)
    pub service_name_snake: String,
    /// サービス名 (`PascalCase`, 自動導出)
    pub service_name_pascal: String,
    /// サービス名 (camelCase, 自動導出)
    pub service_name_camel: String,
    /// 配置階層: system / business / service
    pub tier: String,
    /// 業務領域名 (business Tier 時のみ, それ以外は空文字列)
    pub domain: String,
    /// regions/ からの相対パス (自動導出)
    pub module_path: String,
    /// 言語識別子: go / rust / typescript / dart
    pub language: String,
    /// フレームワーク識別子 (client 時): react / flutter (それ以外は空文字列)
    pub framework: String,
    /// 種別: server / client / library / database
    pub kind: String,
    /// API 方式 (server 時, 後方互換用: `api_styles` の最初の要素): rest / grpc / graphql
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
    /// Helm Chart の Tier 別相対パス (自動導出: "{`service_name`}")
    pub helm_path: String,
    // -----------------------------------------------------------------
    // Terraform 用変数
    // -----------------------------------------------------------------
    /// 環境識別子 (dev / staging / prod)
    pub environment: String,
    /// `PostgreSQL` モジュール有効化
    pub enable_postgresql: bool,
    /// `MySQL` モジュール有効化
    pub enable_mysql: bool,
    /// Kafka モジュール有効化 (Terraform 用; `has_kafka` とは独立)
    pub enable_kafka: bool,
    /// 可観測性スタック有効化
    pub enable_observability: bool,
    /// サービスメッシュ有効化
    pub enable_service_mesh: bool,
    /// Vault 設定有効化
    pub enable_vault: bool,
    /// Harbor プロジェクト管理有効化
    pub enable_harbor: bool,
    // -----------------------------------------------------------------
    // ServiceMesh / DockerCompose 用変数
    // -----------------------------------------------------------------
    /// Namespace (Tier から自動導出: "k1s0-{tier}")
    pub namespace: String,
    /// HTTP サーバーポート
    pub server_port: u16,
    /// gRPC ポート
    pub grpc_port: u16,
    /// サーバー言語 (`DockerCompose` 用: go / rust)
    pub server_language: String,
}

/// `TemplateContext` を構築するためのビルダー。
///
/// CLI の対話フローで収集した最小限の入力値から、
/// テンプレートエンジン仕様の導出ルールに従って全変数を自動計算する。
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct TemplateContextBuilder {
    service_name: String,
    tier: String,
    domain: String,
    language: String,
    framework: String,
    kind: String,
    api_styles: Vec<String>,
    has_database: bool,
    database_type: String,
    has_kafka: bool,
    has_redis: bool,
    docker_registry: String,
    go_module_base: String,
    // Terraform
    environment: String,
    enable_postgresql: bool,
    enable_mysql: bool,
    enable_kafka: bool,
    enable_observability: bool,
    enable_service_mesh: bool,
    enable_vault: bool,
    enable_harbor: bool,
    // ServiceMesh / DockerCompose
    server_port: u16,
    grpc_port: u16,
    server_language: String,
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
            domain: String::new(),
            language: language.to_string(),
            framework: String::new(),
            kind: kind.to_string(),
            api_styles: Vec::new(),
            has_database: false,
            database_type: String::new(),
            has_kafka: false,
            has_redis: false,
            docker_registry: DEFAULT_DOCKER_REGISTRY.to_string(),
            go_module_base: "github.com/org/k1s0".to_string(),
            environment: String::new(),
            enable_postgresql: false,
            enable_mysql: false,
            enable_kafka: false,
            enable_observability: false,
            enable_service_mesh: false,
            enable_vault: false,
            enable_harbor: false,
            server_port: 8080,
            grpc_port: 50051,
            server_language: String::new(),
        }
    }

    /// フレームワークを設定する (client 時)。
    #[must_use]
    pub fn framework(mut self, framework: &str) -> Self {
        self.framework = framework.to_string();
        self
    }

    /// 業務領域名を設定する (business Tier 時)。
    #[must_use]
    pub fn domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_string();
        self
    }

    /// API 方式を設定する (server 時)。単一スタイルの後方互換 API。
    #[must_use]
    pub fn api_style(mut self, api_style: &str) -> Self {
        self.api_styles = vec![api_style.to_string()];
        self
    }

    /// 複数の API 方式を設定する (server 時)。
    #[must_use]
    pub fn api_styles(mut self, styles: Vec<String>) -> Self {
        self.api_styles = styles;
        self
    }

    /// データベース設定を有効にする。
    #[must_use]
    pub fn with_database(mut self, database_type: &str) -> Self {
        self.has_database = true;
        self.database_type = database_type.to_string();
        self
    }

    /// Kafka を有効にする。
    #[must_use]
    pub fn with_kafka(mut self) -> Self {
        self.has_kafka = true;
        self
    }

    /// Redis を有効にする。
    #[must_use]
    pub fn with_redis(mut self) -> Self {
        self.has_redis = true;
        self
    }

    /// Docker レジストリを設定する。
    #[must_use]
    pub fn docker_registry(mut self, registry: &str) -> Self {
        self.docker_registry = registry.to_string();
        self
    }

    /// Go モジュールベースパスを設定する。
    #[must_use]
    pub fn go_module_base(mut self, base: &str) -> Self {
        self.go_module_base = base.to_string();
        self
    }

    /// 環境を設定する (Terraform 用)。
    #[must_use]
    pub fn environment(mut self, env: &str) -> Self {
        self.environment = env.to_string();
        self
    }

    /// Terraform インフラモジュールの有効化フラグを設定する。
    #[must_use]
    pub fn enable_postgresql(mut self) -> Self {
        self.enable_postgresql = true;
        self
    }

    /// `MySQL` モジュールを有効化する。
    #[must_use]
    pub fn enable_mysql(mut self) -> Self {
        self.enable_mysql = true;
        self
    }

    /// Kafka モジュールを有効化する (Terraform 用)。
    #[must_use]
    pub fn enable_kafka(mut self) -> Self {
        self.enable_kafka = true;
        self
    }

    /// 可観測性スタックを有効化する。
    #[must_use]
    pub fn enable_observability(mut self) -> Self {
        self.enable_observability = true;
        self
    }

    /// サービスメッシュを有効化する。
    #[must_use]
    pub fn enable_service_mesh(mut self) -> Self {
        self.enable_service_mesh = true;
        self
    }

    /// Vault を有効化する。
    #[must_use]
    pub fn enable_vault(mut self) -> Self {
        self.enable_vault = true;
        self
    }

    /// Harbor を有効化する。
    #[must_use]
    pub fn enable_harbor(mut self) -> Self {
        self.enable_harbor = true;
        self
    }

    /// HTTP サーバーポートを設定する。
    #[must_use]
    pub fn server_port(mut self, port: u16) -> Self {
        self.server_port = port;
        self
    }

    /// gRPC ポートを設定する。
    #[must_use]
    pub fn grpc_port(mut self, port: u16) -> Self {
        self.grpc_port = port;
        self
    }

    /// サーバー言語を設定する (`DockerCompose` 用)。
    #[must_use]
    pub fn server_language(mut self, lang: &str) -> Self {
        self.server_language = lang.to_string();
        self
    }

    /// `TemplateContext` を構築する。
    ///
    /// 入力値から導出ルールに従って全変数を自動計算する。
    pub fn build(self) -> TemplateContext {
        // ケース変換の導出
        let service_name_snake = self.service_name.to_snake_case();
        let service_name_pascal = self.service_name.to_pascal_case();
        let service_name_camel = self.service_name.to_lower_camel_case();

        // module_path の導出（Tier 別ルール）:
        // service:  "regions/service/{service_name}/{kind}/{language}"
        // system:   "regions/system/{kind}/{language}/{service_name}"
        // business: "regions/business/{domain}/{kind}/{language}/{service_name}"
        let module_path = match self.tier.as_str() {
            "system" => format!(
                "regions/system/{}/{}/{}",
                self.kind, self.language, self.service_name
            ),
            "business" => format!(
                "regions/business/{}/{}/{}/{}",
                self.domain, self.kind, self.language, self.service_name
            ),
            _ => format!(
                "regions/service/{}/{}/{}",
                self.service_name, self.kind, self.language
            ),
        };

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

        // helm_path の導出: service_name をそのまま使用
        let helm_path = self.service_name.clone();

        // api_style: 後方互換のため api_styles の先頭要素を設定
        let api_style = self.api_styles.first().cloned().unwrap_or_default();

        // namespace の導出: "k1s0-{tier}"
        let namespace = format!("k1s0-{}", self.tier);

        // server_language の導出: 明示的に設定されていなければ language を使用
        let server_language = if self.server_language.is_empty() {
            self.language.clone()
        } else {
            self.server_language
        };

        TemplateContext {
            service_name: self.service_name,
            service_name_snake,
            service_name_pascal,
            service_name_camel,
            tier: self.tier,
            domain: self.domain,
            module_path,
            language: self.language,
            framework: self.framework,
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
            helm_path,
            environment: self.environment,
            enable_postgresql: self.enable_postgresql,
            enable_mysql: self.enable_mysql,
            enable_kafka: self.enable_kafka,
            enable_observability: self.enable_observability,
            enable_service_mesh: self.enable_service_mesh,
            enable_vault: self.enable_vault,
            enable_harbor: self.enable_harbor,
            namespace,
            server_port: self.server_port,
            grpc_port: self.grpc_port,
            server_language,
        }
    }
}

impl TemplateContext {
    /// `TemplateContext` を Tera の Context に変換する。
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
        ctx.insert("domain", &self.domain);
        ctx.insert("module_path", &self.module_path);
        ctx.insert("language", &self.language);
        ctx.insert("framework", &self.framework);
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
        ctx.insert("helm_path", &self.helm_path);
        ctx.insert("environment", &self.environment);
        ctx.insert("enable_postgresql", &self.enable_postgresql);
        ctx.insert("enable_mysql", &self.enable_mysql);
        ctx.insert("enable_kafka", &self.enable_kafka);
        ctx.insert("enable_observability", &self.enable_observability);
        ctx.insert("enable_service_mesh", &self.enable_service_mesh);
        ctx.insert("enable_vault", &self.enable_vault);
        ctx.insert("enable_harbor", &self.enable_harbor);
        ctx.insert("namespace", &self.namespace);
        ctx.insert("server_port", &self.server_port);
        ctx.insert("grpc_port", &self.grpc_port);
        ctx.insert("server_language", &self.server_language);
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
        let ctx = TemplateContextBuilder::new("order-api", "service", "rust", "server").build();

        assert_eq!(ctx.service_name, "order-api");
        assert_eq!(ctx.service_name_snake, "order_api");
        assert_eq!(ctx.service_name_pascal, "OrderApi");
        assert_eq!(ctx.service_name_camel, "orderApi");
    }

    #[test]
    fn test_context_name_derivation_user_auth_service() {
        let ctx =
            TemplateContextBuilder::new("user-auth-service", "service", "rust", "server").build();

        assert_eq!(ctx.service_name, "user-auth-service");
        assert_eq!(ctx.service_name_snake, "user_auth_service");
        assert_eq!(ctx.service_name_pascal, "UserAuthService");
        assert_eq!(ctx.service_name_camel, "userAuthService");
    }

    #[test]
    fn test_context_name_derivation_single_word() {
        let ctx = TemplateContextBuilder::new("inventory", "service", "rust", "server").build();

        assert_eq!(ctx.service_name, "inventory");
        assert_eq!(ctx.service_name_snake, "inventory");
        assert_eq!(ctx.service_name_pascal, "Inventory");
        assert_eq!(ctx.service_name_camel, "inventory");
    }

    // =========================================================================
    // domain フィールドのテスト
    // =========================================================================

    #[test]
    fn test_domain_empty_for_service_tier() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();
        assert_eq!(ctx.domain, "");
    }

    #[test]
    fn test_domain_empty_for_system_tier() {
        let ctx = TemplateContextBuilder::new("auth", "system", "rust", "server").build();
        assert_eq!(ctx.domain, "");
    }

    #[test]
    fn test_domain_set_for_business_tier() {
        let ctx = TemplateContextBuilder::new("ledger-api", "business", "rust", "server")
            .domain("accounting")
            .build();
        assert_eq!(ctx.domain, "accounting");
    }

    // =========================================================================
    // module_path の導出テスト
    // =========================================================================

    #[test]
    fn test_context_module_path_service_rust_server() {
        // service tier: regions/service/{service_name}/{kind}/{language}
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();

        assert_eq!(ctx.module_path, "regions/service/order/server/rust");
    }

    #[test]
    fn test_context_module_path_system_library_rust() {
        // system tier: regions/system/{kind}/{language}/{service_name}
        let ctx = TemplateContextBuilder::new("auth", "system", "rust", "library").build();

        assert_eq!(ctx.module_path, "regions/system/library/rust/auth");
    }

    #[test]
    fn test_context_module_path_system_server_rust() {
        // system tier: regions/system/{kind}/{language}/{service_name}
        let ctx = TemplateContextBuilder::new("auth", "system", "rust", "server").build();

        assert_eq!(ctx.module_path, "regions/system/server/rust/auth");
    }

    #[test]
    fn test_context_module_path_business_server_rust() {
        // business tier: regions/business/{domain}/{kind}/{language}/{service_name}
        let ctx = TemplateContextBuilder::new("ledger-api", "business", "rust", "server")
            .domain("accounting")
            .build();

        assert_eq!(
            ctx.module_path,
            "regions/business/accounting/server/rust/ledger-api"
        );
    }

    #[test]
    fn test_context_module_path_business_client_react() {
        // business tier: regions/business/{domain}/{kind}/{language}/{service_name}
        let ctx = TemplateContextBuilder::new("ledger-app", "business", "react", "client")
            .domain("accounting")
            .build();

        assert_eq!(
            ctx.module_path,
            "regions/business/accounting/client/react/ledger-app"
        );
    }

    #[test]
    fn test_context_module_path_business_library_rust() {
        // business tier: regions/business/{domain}/{kind}/{language}/{service_name}
        let ctx = TemplateContextBuilder::new("shared-types", "business", "rust", "library")
            .domain("fa")
            .build();

        assert_eq!(
            ctx.module_path,
            "regions/business/fa/library/rust/shared-types"
        );
    }

    // =========================================================================
    // docker_project の導出テスト
    // =========================================================================

    #[test]
    fn test_context_docker_project_system() {
        let ctx = TemplateContextBuilder::new("auth", "system", "rust", "server").build();

        assert_eq!(ctx.docker_project, "k1s0-system");
    }

    #[test]
    fn test_context_docker_project_business() {
        let ctx = TemplateContextBuilder::new("crm", "business", "rust", "server").build();

        assert_eq!(ctx.docker_project, "k1s0-business");
    }

    #[test]
    fn test_context_docker_project_service() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();

        assert_eq!(ctx.docker_project, "k1s0-service");
    }

    // =========================================================================
    // go_module の導出テスト (Go 言語サポート用、後方互換)
    // =========================================================================

    #[test]
    fn test_go_module_for_go_project() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server").build();

        assert_eq!(
            ctx.go_module,
            "github.com/org/k1s0/regions/service/order/server/go"
        );
    }

    #[test]
    fn test_go_module_empty_for_non_go() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();

        assert_eq!(ctx.go_module, "");
    }

    // =========================================================================
    // rust_crate の導出テスト
    // =========================================================================

    #[test]
    fn test_rust_crate_for_rust_project() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "rust", "server").build();

        assert_eq!(ctx.rust_crate, "order-api");
    }

    #[test]
    fn test_rust_crate_empty_for_non_rust() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server").build();

        assert_eq!(ctx.rust_crate, "");
    }

    // =========================================================================
    // ビルダーのオプション設定テスト
    // =========================================================================

    #[test]
    fn test_builder_api_style() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .api_style("rest")
            .build();

        assert_eq!(ctx.api_style, "rest");
    }

    #[test]
    fn test_builder_with_database() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .with_database("postgresql")
            .build();

        assert!(ctx.has_database);
        assert_eq!(ctx.database_type, "postgresql");
    }

    #[test]
    fn test_builder_without_database() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();

        assert!(!ctx.has_database);
        assert_eq!(ctx.database_type, "");
    }

    #[test]
    fn test_builder_with_kafka() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .with_kafka()
            .build();

        assert!(ctx.has_kafka);
    }

    #[test]
    fn test_builder_with_redis() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .with_redis()
            .build();

        assert!(ctx.has_redis);
    }

    // --- D-04: api_styles Vec ---

    #[test]
    fn test_builder_api_styles_multiple() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .api_styles(vec!["rest".to_string(), "grpc".to_string()])
            .build();

        assert_eq!(ctx.api_styles, vec!["rest".to_string(), "grpc".to_string()]);
        assert_eq!(ctx.api_style, "rest"); // backward compat: first element
    }

    #[test]
    fn test_builder_api_style_backward_compat() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .api_style("grpc")
            .build();

        assert_eq!(ctx.api_styles, vec!["grpc".to_string()]);
        assert_eq!(ctx.api_style, "grpc");
    }

    #[test]
    fn test_builder_api_styles_empty_default() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();

        assert!(ctx.api_styles.is_empty());
        assert_eq!(ctx.api_style, ""); // no styles = empty string
    }

    // --- D-09: go_module_base (Go 言語後方互換) / rust_crate ---

    #[test]
    fn test_builder_go_module_base_custom() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server")
            .go_module_base("github.com/myorg/myrepo")
            .build();

        assert_eq!(
            ctx.go_module,
            "github.com/myorg/myrepo/regions/service/order/server/go"
        );
    }

    #[test]
    fn test_builder_go_module_base_default() {
        let ctx = TemplateContextBuilder::new("order", "service", "go", "server").build();

        assert_eq!(
            ctx.go_module,
            "github.com/org/k1s0/regions/service/order/server/go"
        );
    }

    #[test]
    fn test_rust_crate_system_tier() {
        // system tier, rust: regions/system/server/rust/auth
        let ctx = TemplateContextBuilder::new("auth", "system", "rust", "server").build();

        assert_eq!(ctx.rust_crate, "auth");
        assert_eq!(ctx.module_path, "regions/system/server/rust/auth");
    }

    #[test]
    fn test_rust_crate_business_tier() {
        // business tier, rust: regions/business/accounting/server/rust/ledger-api
        let ctx = TemplateContextBuilder::new("ledger-api", "business", "rust", "server")
            .domain("accounting")
            .build();

        assert_eq!(ctx.rust_crate, "ledger-api");
        assert_eq!(
            ctx.module_path,
            "regions/business/accounting/server/rust/ledger-api"
        );
    }

    #[test]
    fn test_builder_docker_registry_default() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();

        assert_eq!(ctx.docker_registry, "harbor.internal.example.com");
    }

    #[test]
    fn test_builder_docker_registry_custom() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .docker_registry("custom.registry.io")
            .build();

        assert_eq!(ctx.docker_registry, "custom.registry.io");
    }

    #[test]
    fn test_builder_full_options_service() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "rust", "server")
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
        assert_eq!(ctx.domain, "");
        assert_eq!(ctx.module_path, "regions/service/order-api/server/rust");
        assert_eq!(ctx.language, "rust");
        assert_eq!(ctx.kind, "server");
        assert_eq!(ctx.api_style, "rest");
        assert!(ctx.has_database);
        assert_eq!(ctx.database_type, "postgresql");
        assert!(ctx.has_kafka);
        assert!(ctx.has_redis);
        assert_eq!(ctx.go_module, "");
        assert_eq!(ctx.rust_crate, "order-api");
        assert_eq!(ctx.docker_registry, "harbor.internal.example.com");
        assert_eq!(ctx.docker_project, "k1s0-service");
        assert_eq!(ctx.helm_path, "order-api");
    }

    #[test]
    fn test_builder_full_options_business() {
        let ctx = TemplateContextBuilder::new("ledger-api", "business", "rust", "server")
            .domain("accounting")
            .api_style("rest")
            .with_database("postgresql")
            .build();

        assert_eq!(ctx.service_name, "ledger-api");
        assert_eq!(ctx.domain, "accounting");
        assert_eq!(
            ctx.module_path,
            "regions/business/accounting/server/rust/ledger-api"
        );
        assert_eq!(ctx.rust_crate, "ledger-api");
        assert_eq!(ctx.docker_project, "k1s0-business");
    }

    // =========================================================================
    // to_tera_context のテスト
    // =========================================================================

    #[test]
    fn test_to_tera_context_contains_all_fields() {
        let ctx = TemplateContextBuilder::new("order-api", "service", "rust", "server")
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
        assert_eq!(json["domain"], "");
        assert_eq!(json["module_path"], "regions/service/order-api/server/rust");
        assert_eq!(json["language"], "rust");
        assert_eq!(json["kind"], "server");
        assert_eq!(json["api_style"], "rest");
        assert_eq!(json["api_styles"], serde_json::json!(["rest"]));
        assert_eq!(json["has_database"], true);
        assert_eq!(json["database_type"], "postgresql");
        assert_eq!(json["has_kafka"], true);
        assert_eq!(json["has_redis"], true);
        assert_eq!(json["go_module"], "");
        assert_eq!(json["rust_crate"], "order-api");
        assert_eq!(json["docker_registry"], "harbor.internal.example.com");
        assert_eq!(json["docker_project"], "k1s0-service");
        assert_eq!(json["helm_path"], "order-api");
    }

    #[test]
    fn test_to_tera_context_business_with_domain() {
        let ctx = TemplateContextBuilder::new("ledger-api", "business", "rust", "server")
            .domain("accounting")
            .build();

        let tera_ctx = ctx.to_tera_context();
        let json = tera_ctx.into_json();

        assert_eq!(json["domain"], "accounting");
        assert_eq!(
            json["module_path"],
            "regions/business/accounting/server/rust/ledger-api"
        );
    }

    #[test]
    fn test_to_tera_context_flat_access() {
        // テンプレートで {{ service_name }} のようにフラットアクセスできることを検証
        let ctx = TemplateContextBuilder::new("order-api", "service", "rust", "server").build();

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
        let ctx = TemplateContextBuilder::new("order-api", "service", "rust", "server")
            .api_style("rest")
            .build();

        let json = serde_json::to_value(&ctx).unwrap();
        assert_eq!(json["service_name"], "order-api");
        assert_eq!(json["service_name_snake"], "order_api");
        assert_eq!(json["tier"], "service");
        assert_eq!(json["kind"], "server");
    }

    // =========================================================================
    // Terraform 用変数のテスト
    // =========================================================================

    #[test]
    fn test_builder_environment() {
        let ctx = TemplateContextBuilder::new("k1s0", "system", "rust", "terraform")
            .environment("prod")
            .build();
        assert_eq!(ctx.environment, "prod");
    }

    #[test]
    fn test_builder_environment_default_empty() {
        let ctx = TemplateContextBuilder::new("k1s0", "system", "rust", "terraform").build();
        assert_eq!(ctx.environment, "");
    }

    #[test]
    fn test_builder_enable_flags() {
        let ctx = TemplateContextBuilder::new("k1s0", "system", "rust", "terraform")
            .environment("dev")
            .enable_postgresql()
            .enable_kafka()
            .enable_observability()
            .enable_vault()
            .build();

        assert!(ctx.enable_postgresql);
        assert!(!ctx.enable_mysql);
        assert!(ctx.enable_kafka);
        assert!(ctx.enable_observability);
        assert!(!ctx.enable_service_mesh);
        assert!(ctx.enable_vault);
        assert!(!ctx.enable_harbor);
    }

    #[test]
    fn test_builder_enable_all_flags() {
        let ctx = TemplateContextBuilder::new("k1s0", "system", "rust", "terraform")
            .enable_postgresql()
            .enable_mysql()
            .enable_kafka()
            .enable_observability()
            .enable_service_mesh()
            .enable_vault()
            .enable_harbor()
            .build();

        assert!(ctx.enable_postgresql);
        assert!(ctx.enable_mysql);
        assert!(ctx.enable_kafka);
        assert!(ctx.enable_observability);
        assert!(ctx.enable_service_mesh);
        assert!(ctx.enable_vault);
        assert!(ctx.enable_harbor);
    }

    #[test]
    fn test_builder_enable_flags_default_false() {
        let ctx = TemplateContextBuilder::new("k1s0", "system", "rust", "terraform").build();

        assert!(!ctx.enable_postgresql);
        assert!(!ctx.enable_mysql);
        assert!(!ctx.enable_kafka);
        assert!(!ctx.enable_observability);
        assert!(!ctx.enable_service_mesh);
        assert!(!ctx.enable_vault);
        assert!(!ctx.enable_harbor);
    }

    // =========================================================================
    // namespace の自動導出テスト
    // =========================================================================

    #[test]
    fn test_namespace_derived_from_tier_system() {
        let ctx = TemplateContextBuilder::new("auth", "system", "rust", "server").build();
        assert_eq!(ctx.namespace, "k1s0-system");
    }

    #[test]
    fn test_namespace_derived_from_tier_business() {
        let ctx = TemplateContextBuilder::new("ledger", "business", "rust", "server").build();
        assert_eq!(ctx.namespace, "k1s0-business");
    }

    #[test]
    fn test_namespace_derived_from_tier_service() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();
        assert_eq!(ctx.namespace, "k1s0-service");
    }

    // =========================================================================
    // server_port / grpc_port のテスト
    // =========================================================================

    #[test]
    fn test_server_port_default() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();
        assert_eq!(ctx.server_port, 8080);
    }

    #[test]
    fn test_server_port_custom() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .server_port(80)
            .build();
        assert_eq!(ctx.server_port, 80);
    }

    #[test]
    fn test_grpc_port_default() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();
        assert_eq!(ctx.grpc_port, 50051);
    }

    #[test]
    fn test_grpc_port_custom() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server")
            .grpc_port(9090)
            .build();
        assert_eq!(ctx.grpc_port, 9090);
    }

    // =========================================================================
    // server_language のテスト
    // =========================================================================

    #[test]
    fn test_server_language_defaults_to_language() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "server").build();
        assert_eq!(ctx.server_language, "rust");
    }

    #[test]
    fn test_server_language_custom() {
        let ctx = TemplateContextBuilder::new("order", "service", "rust", "docker-compose")
            .server_language("rust")
            .build();
        assert_eq!(ctx.server_language, "rust");
    }

    // =========================================================================
    // to_tera_context に新フィールドが含まれることのテスト
    // =========================================================================

    #[test]
    fn test_to_tera_context_contains_new_fields() {
        let ctx = TemplateContextBuilder::new("k1s0", "system", "rust", "terraform")
            .environment("prod")
            .enable_postgresql()
            .enable_observability()
            .enable_vault()
            .server_port(80)
            .grpc_port(9090)
            .build();

        let tera_ctx = ctx.to_tera_context();
        let json = tera_ctx.into_json();

        assert_eq!(json["environment"], "prod");
        assert_eq!(json["enable_postgresql"], true);
        assert_eq!(json["enable_mysql"], false);
        assert_eq!(json["enable_observability"], true);
        assert_eq!(json["enable_vault"], true);
        assert_eq!(json["enable_harbor"], false);
        assert_eq!(json["namespace"], "k1s0-system");
        assert_eq!(json["server_port"], 80);
        assert_eq!(json["grpc_port"], 9090);
        assert_eq!(json["server_language"], "rust");
    }
}
