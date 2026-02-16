/// サーバーテンプレートのレンダリング統合テスト。
///
/// 実際の CLI/templates/server/{go,rust}/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書
/// (docs/テンプレート仕様-サーバー.md) と一致することを検証する。
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

fn render_server(
    lang: &str,
    api_style: &str,
    has_database: bool,
    database_type: &str,
    has_kafka: bool,
    has_redis: bool,
) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut builder = TemplateContextBuilder::new("order-api", "service", lang, "server")
        .api_style(api_style);

    if has_database {
        builder = builder.with_database(database_type);
    }
    if has_kafka {
        builder = builder.with_kafka();
    }
    if has_redis {
        builder = builder.with_redis();
    }

    let ctx = builder.build();
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
// Go サーバー: REST + PostgreSQL + Kafka + Redis
// =========================================================================

#[test]
fn test_go_server_rest_full_stack_file_list() {
    let (tmp, names) = render_server("go", "rest", true, "postgresql", true, false);

    // 必須ファイルの存在確認
    assert!(names.iter().any(|n| n == "go.mod"), "go.mod missing");
    assert!(names.iter().any(|n| n == "cmd/main.go"), "cmd/main.go missing");
    assert!(
        names.iter().any(|n| n == "internal/domain/model/entity.go"),
        "entity.go missing"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "internal/domain/repository/repository.go"),
        "repository.go missing"
    );
    assert!(
        names.iter().any(|n| n == "internal/usecase/usecase.go"),
        "usecase.go missing"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "internal/adapter/handler/rest_handler.go"),
        "rest_handler.go missing"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "internal/infra/persistence/db.go"),
        "db.go missing"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "internal/infra/persistence/repository.go"),
        "persistence/repository.go missing"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "internal/infra/messaging/kafka.go"),
        "kafka.go missing"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "internal/infra/config/config.go"),
        "config.go missing"
    );
    assert!(
        names.iter().any(|n| n == "config/config.yaml"),
        "config.yaml missing"
    );
    assert!(
        names.iter().any(|n| n == "api/openapi/openapi.yaml"),
        "openapi.yaml missing"
    );
    assert!(names.iter().any(|n| n == "Dockerfile"), "Dockerfile missing");

    // テストファイルの存在確認
    assert!(
        names.iter().any(|n| n == "internal/usecase/usecase_test.go"),
        "usecase_test.go missing"
    );
    assert!(
        names.iter().any(|n| n == "internal/adapter/handler/handler_test.go"),
        "handler_test.go missing"
    );
    assert!(
        names.iter().any(|n| n == "internal/infra/persistence/repository_test.go"),
        "repository_test.go missing"
    );

    // REST の場合 gRPC / GraphQL は除外される
    assert!(
        !names
            .iter()
            .any(|n| n.contains("grpc_handler")),
        "grpc_handler should not exist for REST"
    );
    assert!(
        !names
            .iter()
            .any(|n| n.contains("graphql_resolver")),
        "graphql_resolver should not exist for REST"
    );
    assert!(
        !names
            .iter()
            .any(|n| n.contains("service.proto")),
        "service.proto should not exist for REST"
    );
    assert!(!names.iter().any(|n| n.contains("buf.yaml")), "buf.yaml should not exist for REST");
    assert!(!names.iter().any(|n| n.contains("buf.gen")), "buf.gen.yaml should not exist for REST");
    assert!(!names.iter().any(|n| n.contains("schema.graphql")), "schema.graphql should not exist for REST");
    assert!(!names.iter().any(|n| n.contains("gqlgen.yml")), "gqlgen.yml should not exist for REST");

    // go.mod の内容検証
    let go_mod = read_output(&tmp, "go.mod");
    assert!(go_mod.contains("module github.com/org/k1s0/regions/service/order-api/server/go"));
    assert!(go_mod.contains("github.com/gin-gonic/gin"));
    assert!(go_mod.contains("github.com/oapi-codegen/oapi-codegen/v2"));
    assert!(go_mod.contains("github.com/jmoiron/sqlx"));
    assert!(go_mod.contains("github.com/lib/pq"));
    assert!(go_mod.contains("github.com/segmentio/kafka-go"));
    // grpc/graphql の依存は含まれない
    assert!(!go_mod.contains("google.golang.org/grpc"));
    assert!(!go_mod.contains("github.com/99designs/gqlgen"));
}

#[test]
fn test_go_server_rest_main_go_content() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", true, false);
    let content = read_output(&tmp, "cmd/main.go");

    // import文の検証
    assert!(content.contains("\"github.com/gin-gonic/gin\""));
    assert!(content.contains("go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin"));
    assert!(content.contains("/internal/adapter/handler\""));
    assert!(content.contains("/internal/infra/config\""));
    assert!(content.contains("/internal/infra/persistence\""));
    assert!(content.contains("/internal/infra/messaging\""));
    assert!(content.contains("/internal/usecase\""));

    // DB 初期化
    assert!(content.contains("persistence.NewDB(cfg.Database)"));
    assert!(content.contains("defer db.Close()"));

    // Kafka
    assert!(content.contains("messaging.NewProducer(cfg.Kafka)"));
    assert!(content.contains("defer producer.Close()"));

    // DI: repo -> uc -> handler
    assert!(content.contains("persistence.NewRepository(db)"));
    assert!(content.contains("usecase.NewOrderApiUseCase("));
    assert!(content.contains("handler.NewHandler(uc)"));

    // healthz / readyz
    assert!(content.contains("r.GET(\"/healthz\""));
    assert!(content.contains("r.GET(\"/readyz\""));
    assert!(content.contains("db.PingContext"));

    // graceful shutdown
    assert!(content.contains("signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)"));
    assert!(content.contains("srv.Shutdown(ctx)"));
}

#[test]
fn test_go_server_rest_entity() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/domain/model/entity.go");

    assert!(content.contains("package model"));
    assert!(content.contains("import \"time\""));
    assert!(content.contains("type OrderApiEntity struct {"));
    assert!(content.contains("ID          string    `json:\"id\" db:\"id\"`"));
    assert!(content.contains("Name        string    `json:\"name\" db:\"name\" validate:\"required,max=255\"`"));
    assert!(content.contains("CreatedAt   time.Time `json:\"created_at\" db:\"created_at\"`"));
}

#[test]
fn test_go_server_rest_repository() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/domain/repository/repository.go");

    assert!(content.contains("package repository"));
    assert!(content.contains("//go:generate mockgen"));
    assert!(content.contains("type OrderApiRepository interface {"));
    assert!(content.contains("FindByID(ctx context.Context, id string) (*model.OrderApiEntity, error)"));
    assert!(content.contains("FindAll(ctx context.Context) ([]*model.OrderApiEntity, error)"));
    assert!(content.contains("Create(ctx context.Context, entity *model.OrderApiEntity) error"));
    assert!(content.contains("Update(ctx context.Context, entity *model.OrderApiEntity) error"));
    assert!(content.contains("Delete(ctx context.Context, id string) error"));
}

#[test]
fn test_go_server_rest_usecase() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/usecase/usecase.go");

    assert!(content.contains("package usecase"));
    assert!(content.contains("type OrderApiUseCase struct {"));
    assert!(content.contains("repo repository.OrderApiRepository"));
    assert!(content.contains("func NewOrderApiUseCase("));
    assert!(content.contains("func (uc *OrderApiUseCase) GetByID(ctx context.Context, id string) (*model.OrderApiEntity, error)"));
    assert!(content.contains("return uc.repo.FindByID(ctx, id)"));
    assert!(content.contains("func (uc *OrderApiUseCase) Create(ctx context.Context, entity *model.OrderApiEntity) error"));
    assert!(content.contains("return uc.repo.Create(ctx, entity)"));
}

#[test]
fn test_go_server_rest_handler() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/adapter/handler/rest_handler.go");

    assert!(content.contains("package handler"));
    assert!(content.contains("var validate = validator.New()"));
    assert!(content.contains("type Handler struct {"));
    assert!(content.contains("uc *usecase.OrderApiUseCase"));
    assert!(content.contains("func NewHandler(uc *usecase.OrderApiUseCase) *Handler"));
    assert!(content.contains("func (h *Handler) RegisterRoutes(r *gin.Engine)"));
    assert!(content.contains("v1.GET(\"/order-api\""));
    assert!(content.contains("v1.GET(\"/order-api/:id\""));
    assert!(content.contains("v1.POST(\"/order-api\""));

    // ErrorResponse: D-007
    assert!(content.contains("type ErrorResponse struct {"));
    assert!(content.contains("Code    string `json:\"code\"`"));
    assert!(content.contains("Message string `json:\"message\"`"));

    // List, GetByID, Create endpoints
    assert!(content.contains("func (h *Handler) List(c *gin.Context)"));
    assert!(content.contains("func (h *Handler) GetByID(c *gin.Context)"));
    assert!(content.contains("func (h *Handler) Create(c *gin.Context)"));
    assert!(content.contains("\"INTERNAL_ERROR\""));
    assert!(content.contains("\"NOT_FOUND\""));
    assert!(content.contains("\"INVALID_REQUEST\""));
    assert!(content.contains("\"VALIDATION_ERROR\""));
}

#[test]
fn test_go_server_rest_persistence_db() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/infra/persistence/db.go");

    assert!(content.contains("package persistence"));
    assert!(content.contains("\"github.com/jmoiron/sqlx\""));
    assert!(content.contains("_ \"github.com/lib/pq\""));
    assert!(content.contains("func NewDB(cfg config.DatabaseConfig) (*sqlx.DB, error)"));
    assert!(content.contains("sqlx.Connect(\"postgres\", dsn)"));
    assert!(content.contains("db.SetMaxOpenConns(cfg.Pool.MaxOpen)"));
    assert!(content.contains("db.SetMaxIdleConns(cfg.Pool.MaxIdle)"));
    assert!(content.contains("func buildDSN(cfg config.DatabaseConfig) string"));
    assert!(content.contains("host=%s port=%d user=%s password=%s dbname=%s sslmode=%s"));
}

#[test]
fn test_go_server_rest_persistence_repository() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/infra/persistence/repository.go");

    assert!(content.contains("package persistence"));
    assert!(content.contains("type orderApiRepository struct {"));
    assert!(content.contains("func NewRepository(db *sqlx.DB) repository.OrderApiRepository"));
    assert!(content.contains("func (r *orderApiRepository) FindByID(ctx context.Context, id string)"));
    assert!(content.contains("func (r *orderApiRepository) FindAll(ctx context.Context)"));
    assert!(content.contains("func (r *orderApiRepository) Create(ctx context.Context, entity *model.OrderApiEntity)"));
    assert!(content.contains("func (r *orderApiRepository) Update(ctx context.Context, entity *model.OrderApiEntity)"));
    assert!(content.contains("func (r *orderApiRepository) Delete(ctx context.Context, id string)"));
}

#[test]
fn test_go_server_rest_kafka() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", true, false);
    let content = read_output(&tmp, "internal/infra/messaging/kafka.go");

    assert!(content.contains("package messaging"));
    assert!(content.contains("type Producer struct {"));
    assert!(content.contains("func NewProducer(cfg config.KafkaConfig) *Producer"));
    assert!(content.contains("func (p *Producer) Publish(ctx context.Context, topic string, key, value []byte) error"));
    assert!(content.contains("func (p *Producer) Close()"));
    assert!(content.contains("type Consumer struct {"));
    assert!(content.contains("func NewConsumer(cfg config.KafkaConfig, topic, groupID string) *Consumer"));
    assert!(content.contains("func (c *Consumer) Consume(ctx context.Context, handler func(kafka.Message) error) error"));
    assert!(content.contains("func (c *Consumer) Close()"));
    // 命名規則コメント
    assert!(content.contains("k1s0.{tier}.{domain}.{event-type}.{version}"));
    assert!(content.contains("{service-name}.{purpose}"));
}

#[test]
fn test_go_server_rest_config() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", true, true);
    let content = read_output(&tmp, "internal/infra/config/config.go");

    assert!(content.contains("package config"));
    assert!(content.contains("type Config struct {"));
    assert!(content.contains("App           AppConfig"));
    assert!(content.contains("Server        ServerConfig"));
    assert!(content.contains("*DatabaseConfig"));
    assert!(content.contains("*KafkaConfig"));
    assert!(content.contains("*RedisConfig"));
    assert!(content.contains("Observability ObservabilityConfig"));
    assert!(content.contains("func Load(path string) (*Config, error)"));
}

#[test]
fn test_go_server_rest_config_yaml() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", true, true);
    let content = read_output(&tmp, "config/config.yaml");

    assert!(content.contains("name: \"order-api\""));
    assert!(content.contains("port: 8080"));
    assert!(content.contains("database:"));
    assert!(content.contains("host: \"localhost\""));
    assert!(content.contains("port: 5432"));
    assert!(content.contains("ssl_mode: \"disable\""));
    assert!(content.contains("kafka:"));
    assert!(content.contains("kafka-0.messaging.svc.cluster.local:9092"));
    assert!(content.contains("redis:"));
    assert!(content.contains("host: \"redis.k1s0-system.svc.cluster.local\""));
    assert!(content.contains("observability:"));
    assert!(content.contains("endpoint: \"jaeger.observability.svc.cluster.local:4317\""));
}

#[test]
fn test_go_server_rest_openapi() {
    let (tmp, _) = render_server("go", "rest", false, "", false, false);
    let content = read_output(&tmp, "api/openapi/openapi.yaml");

    assert!(content.contains("openapi: \"3.0.3\""));
    assert!(content.contains("title: \"OrderApi API\""));
    assert!(content.contains("/api/v1/order-api:"));
    assert!(content.contains("/api/v1/order-api/{id}:"));
    assert!(content.contains("operationId: \"listOrderApi\""));
    assert!(content.contains("operationId: \"createOrderApi\""));
    assert!(content.contains("operationId: \"getOrderApi\""));
    assert!(content.contains("$ref: \"#/components/schemas/OrderApi\""));
    assert!(content.contains("$ref: \"#/components/schemas/CreateOrderApiRequest\""));
    assert!(content.contains("$ref: \"#/components/schemas/ErrorResponse\""));
}

#[test]
fn test_go_server_rest_dockerfile() {
    let (tmp, _) = render_server("go", "rest", false, "", false, false);
    let content = read_output(&tmp, "Dockerfile");

    assert!(content.contains("FROM golang:1.23-bookworm AS builder"));
    assert!(content.contains("CGO_ENABLED=0 GOOS=linux go build -trimpath"));
    assert!(content.contains("FROM gcr.io/distroless/static-debian12"));
    assert!(content.contains("EXPOSE 8080"));
    assert!(content.contains("USER nonroot:nonroot"));
    assert!(content.contains("ENTRYPOINT [\"/server\"]"));
}

// =========================================================================
// Go サーバー: gRPC
// =========================================================================

#[test]
fn test_go_server_grpc_file_list() {
    let (_, names) = render_server("go", "grpc", false, "", false, false);

    assert!(names.iter().any(|n| n.contains("grpc_handler.go")));
    assert!(names.iter().any(|n| n.contains("service.proto")));
    assert!(names.iter().any(|n| n.contains("buf.yaml")), "buf.yaml missing for gRPC");
    assert!(names.iter().any(|n| n.contains("buf.gen")), "buf.gen.yaml missing for gRPC");
    assert!(names.iter().any(|n| n.contains("usecase_test.go")), "usecase_test.go missing");
    assert!(names.iter().any(|n| n.contains("handler_test.go")), "handler_test.go missing");
    assert!(!names.iter().any(|n| n.contains("rest_handler.go")));
    assert!(!names.iter().any(|n| n.contains("openapi")));
    assert!(!names.iter().any(|n| n.contains("graphql_resolver")));
    assert!(!names.iter().any(|n| n.contains("schema.graphql")));
    assert!(!names.iter().any(|n| n.contains("gqlgen.yml")));
}

#[test]
fn test_go_server_grpc_handler() {
    let (tmp, _) = render_server("go", "grpc", false, "", false, false);
    let content = read_output(&tmp, "internal/adapter/handler/grpc_handler.go");

    assert!(content.contains("package handler"));
    assert!(content.contains("type GRPCHandler struct {"));
    assert!(content.contains("pb.UnimplementedOrderApiServiceServer"));
    assert!(content.contains("func NewGRPCHandler(uc *usecase.OrderApiUseCase) *GRPCHandler"));
    assert!(content.contains("func (h *GRPCHandler) GetOrderApi(ctx context.Context, req *pb.GetOrderApiRequest)"));
    assert!(content.contains("codes.Internal"));
    assert!(content.contains("codes.NotFound"));
}

#[test]
fn test_go_server_grpc_proto() {
    let (tmp, _) = render_server("go", "grpc", false, "", false, false);
    let content = read_output(&tmp, "api/proto/service.proto");

    assert!(content.contains("syntax = \"proto3\";"));
    assert!(content.contains("package k1s0.service.order_api.v1;"));
    assert!(content.contains("option go_package = \"github.com/org/k1s0/regions/service/order-api/server/go/api/proto/gen\""));
    assert!(content.contains("service OrderApiService {"));
    assert!(content.contains("rpc GetOrderApi"));
    assert!(content.contains("rpc ListOrderApi"));
    assert!(content.contains("rpc CreateOrderApi"));
    assert!(content.contains("message GetOrderApiRequest {"));
    assert!(content.contains("message GetOrderApiResponse {"));
    assert!(content.contains("message ListOrderApiRequest {"));
    assert!(content.contains("message ListOrderApiResponse {"));
    assert!(content.contains("message CreateOrderApiRequest {"));
    assert!(content.contains("message CreateOrderApiResponse {"));
}

#[test]
fn test_go_server_grpc_go_mod() {
    let (tmp, _) = render_server("go", "grpc", false, "", false, false);
    let content = read_output(&tmp, "go.mod");

    assert!(content.contains("google.golang.org/grpc"));
    assert!(content.contains("google.golang.org/protobuf"));
    assert!(!content.contains("github.com/oapi-codegen"));
    assert!(!content.contains("github.com/99designs/gqlgen"));
}

// =========================================================================
// Go サーバー: GraphQL
// =========================================================================

#[test]
fn test_go_server_graphql_file_list() {
    let (_, names) = render_server("go", "graphql", false, "", false, false);

    assert!(names.iter().any(|n| n.contains("graphql_resolver.go")));
    assert!(names.iter().any(|n| n.contains("schema.graphql")), "schema.graphql missing for GraphQL");
    assert!(names.iter().any(|n| n.contains("gqlgen.yml")), "gqlgen.yml missing for GraphQL");
    assert!(names.iter().any(|n| n.contains("usecase_test.go")), "usecase_test.go missing");
    assert!(names.iter().any(|n| n.contains("handler_test.go")), "handler_test.go missing");
    assert!(!names.iter().any(|n| n.contains("rest_handler.go")));
    assert!(!names.iter().any(|n| n.contains("grpc_handler.go")));
    assert!(!names.iter().any(|n| n.contains("openapi")));
    assert!(!names.iter().any(|n| n.contains("service.proto")));
    assert!(!names.iter().any(|n| n.contains("buf.yaml")));
    assert!(!names.iter().any(|n| n.contains("buf.gen")));
}

#[test]
fn test_go_server_graphql_resolver() {
    let (tmp, _) = render_server("go", "graphql", false, "", false, false);
    let content = read_output(&tmp, "internal/adapter/handler/graphql_resolver.go");

    assert!(content.contains("package handler"));
    assert!(content.contains("type Resolver struct {"));
    assert!(content.contains("func NewResolver(uc *usecase.OrderApiUseCase) *Resolver"));
    assert!(content.contains("func (r *Resolver) Query() QueryResolver"));
    assert!(content.contains("type queryResolver struct{ *Resolver }"));
    assert!(content.contains("func (r *queryResolver) OrderApi(ctx context.Context, id string) (*model.OrderApiEntity, error)"));
    assert!(content.contains("func (r *queryResolver) OrderApiList(ctx context.Context) ([]*model.OrderApiEntity, error)"));
}

#[test]
fn test_go_server_graphql_go_mod() {
    let (tmp, _) = render_server("go", "graphql", false, "", false, false);
    let content = read_output(&tmp, "go.mod");

    assert!(content.contains("github.com/99designs/gqlgen"));
    assert!(content.contains("github.com/vektah/gqlparser/v2"));
    assert!(!content.contains("github.com/oapi-codegen"));
    assert!(!content.contains("google.golang.org/grpc"));
}

// =========================================================================
// Go サーバー: DB なし
// =========================================================================

#[test]
fn test_go_server_no_database() {
    let (tmp, names) = render_server("go", "rest", false, "", false, false);

    // DB 関連ファイルは除外
    assert!(!names.iter().any(|n| n.contains("persistence")));

    // usecase は DB なしパスで生成
    let usecase = read_output(&tmp, "internal/usecase/usecase.go");
    assert!(!usecase.contains("repo repository"));
    assert!(usecase.contains("// TODO:"));

    // main.go に persistence import がない
    let main = read_output(&tmp, "cmd/main.go");
    assert!(!main.contains("persistence"));

    // config に database セクションがない
    let config = read_output(&tmp, "config/config.yaml");
    assert!(!config.contains("database:"));

    // go.mod に sqlx がない
    let go_mod = read_output(&tmp, "go.mod");
    assert!(!go_mod.contains("github.com/jmoiron/sqlx"));
}

// =========================================================================
// Go サーバー: MySQL
// =========================================================================

#[test]
fn test_go_server_mysql() {
    let (tmp, _) = render_server("go", "rest", true, "mysql", false, false);

    let go_mod = read_output(&tmp, "go.mod");
    assert!(go_mod.contains("github.com/go-sql-driver/mysql"));
    assert!(!go_mod.contains("github.com/lib/pq"));

    let db = read_output(&tmp, "internal/infra/persistence/db.go");
    assert!(db.contains("_ \"github.com/go-sql-driver/mysql\""));
    assert!(db.contains("sqlx.Connect(\"mysql\", dsn)"));
    assert!(db.contains("parseTime=true&charset=utf8mb4"));

    let config_yaml = read_output(&tmp, "config/config.yaml");
    assert!(config_yaml.contains("port: 3306"));
    assert!(!config_yaml.contains("ssl_mode"));
}

// =========================================================================
// Go サーバー: SQLite
// =========================================================================

#[test]
fn test_go_server_sqlite() {
    let (tmp, _) = render_server("go", "rest", true, "sqlite", false, false);

    let go_mod = read_output(&tmp, "go.mod");
    assert!(go_mod.contains("github.com/mattn/go-sqlite3"));
    assert!(!go_mod.contains("github.com/lib/pq"));

    let db = read_output(&tmp, "internal/infra/persistence/db.go");
    assert!(db.contains("_ \"github.com/mattn/go-sqlite3\""));
    assert!(db.contains("sqlx.Connect(\"sqlite3\", dsn)"));
}

// =========================================================================
// Rust サーバー: REST + PostgreSQL + Kafka + Redis
// =========================================================================

#[test]
fn test_rust_server_rest_full_stack_file_list() {
    let (_, names) = render_server("rust", "rest", true, "postgresql", true, false);

    assert!(names.iter().any(|n| n == "Cargo.toml"), "Cargo.toml missing");
    assert!(names.iter().any(|n| n == "src/main.rs"), "src/main.rs missing");
    assert!(names.iter().any(|n| n == "src/domain/mod.rs"), "domain/mod.rs missing");
    assert!(names.iter().any(|n| n == "src/domain/model.rs"), "domain/model.rs missing");
    assert!(
        names.iter().any(|n| n == "src/domain/repository.rs"),
        "domain/repository.rs missing"
    );
    assert!(names.iter().any(|n| n == "src/usecase/mod.rs"), "usecase/mod.rs missing");
    assert!(
        names.iter().any(|n| n == "src/usecase/service.rs"),
        "usecase/service.rs missing"
    );
    assert!(names.iter().any(|n| n == "src/adapter/mod.rs"), "adapter/mod.rs missing");
    assert!(
        names
            .iter()
            .any(|n| n == "src/adapter/handler/mod.rs"),
        "adapter/handler/mod.rs missing"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "src/adapter/handler/rest.rs"),
        "adapter/handler/rest.rs missing"
    );
    assert!(names.iter().any(|n| n == "src/infra/mod.rs"), "infra/mod.rs missing");
    assert!(
        names.iter().any(|n| n == "src/infra/config.rs"),
        "infra/config.rs missing"
    );
    assert!(
        names.iter().any(|n| n == "src/infra/persistence.rs"),
        "infra/persistence.rs missing"
    );
    assert!(
        names.iter().any(|n| n == "src/infra/messaging.rs"),
        "infra/messaging.rs missing"
    );
    assert!(names.iter().any(|n| n == "config/config.yaml"), "config.yaml missing");
    assert!(names.iter().any(|n| n == "Dockerfile"), "Dockerfile missing");

    // テストファイルの存在確認
    assert!(
        names.iter().any(|n| n == "tests/integration_test.rs"),
        "tests/integration_test.rs missing"
    );

    // REST 以外のハンドラは除外
    assert!(!names.iter().any(|n| n.contains("grpc.rs")));
    assert!(!names.iter().any(|n| n.contains("graphql.rs")));
    assert!(!names.iter().any(|n| n.contains("buf.yaml")));
    assert!(!names.iter().any(|n| n.contains("build.rs")));
    assert!(!names.iter().any(|n| n.contains("schema.graphql")));
    assert!(!names.iter().any(|n| n.contains("gqlgen.yml")));
}

#[test]
fn test_rust_server_rest_cargo_toml() {
    let (tmp, _) = render_server("rust", "rest", true, "postgresql", true, true);
    let content = read_output(&tmp, "Cargo.toml");

    assert!(content.contains("name = \"order-api\""));
    assert!(content.contains("axum = \"0.7\""));
    assert!(content.contains("tokio = { version = \"1\", features = [\"full\"] }"));
    assert!(content.contains("serde = { version = \"1\", features = [\"derive\"] }"));
    assert!(content.contains("tracing = \"0.1\""));
    assert!(content.contains("tower-http = { version = \"0.6\", features = [\"cors\", \"trace\"] }"));
    assert!(content.contains("utoipa = { version = \"5\", features = [\"axum_extras\"] }"));
    assert!(content.contains("sqlx = { version = \"0.8\", features = [\"runtime-tokio-rustls\", \"postgresql\"] }"));
    assert!(content.contains("rdkafka = { version = \"0.36\", features = [\"cmake-build\"] }"));
    assert!(content.contains("redis = { version = \"0.27\", features = [\"tokio-comp\"] }"));
    assert!(content.contains("[dev-dependencies]"));
    assert!(content.contains("mockall = \"0.13\""));
    // gRPC 依存は含まれない
    assert!(!content.contains("tonic"));
    assert!(!content.contains("prost"));
    assert!(!content.contains("async-graphql"));
}

#[test]
fn test_rust_server_rest_main_rs() {
    let (tmp, _) = render_server("rust", "rest", true, "postgresql", true, false);
    let content = read_output(&tmp, "src/main.rs");

    assert!(content.contains("use axum::{routing::get, Json, Router};"));
    assert!(content.contains("use tower_http::cors::CorsLayer;"));
    assert!(content.contains("use tower_http::trace::TraceLayer;"));
    assert!(content.contains("mod adapter;"));
    assert!(content.contains("mod domain;"));
    assert!(content.contains("mod infra;"));
    assert!(content.contains("mod usecase;"));
    assert!(content.contains("use infra::config::Config;"));

    // DB
    assert!(content.contains("infra::persistence::create_pool(&config.database)"));
    assert!(content.contains("infra::persistence::Repository::new(pool.clone())"));

    // Kafka
    assert!(content.contains("infra::messaging::Producer::new(&config.kafka)"));

    // DI
    assert!(content.contains("usecase::OrderApiUseCase::new("));
    assert!(content.contains("adapter::handler::AppHandler::new(uc)"));

    // healthz / readyz
    assert!(content.contains("\"/healthz\""));
    assert!(content.contains("\"/readyz\""));
    assert!(content.contains("sqlx::query(\"SELECT 1\").execute(&pool)"));

    // graceful shutdown
    assert!(content.contains("shutdown_signal()"));
    assert!(content.contains("async fn shutdown_signal()"));
    assert!(content.contains("tokio::signal::ctrl_c()"));
}

#[test]
fn test_rust_server_rest_domain_model() {
    let (tmp, _) = render_server("rust", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "src/domain/model.rs");

    assert!(content.contains("use serde::{Deserialize, Serialize};"));
    assert!(content.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"));
    assert!(content.contains("#[derive(sqlx::FromRow)]"));
    assert!(content.contains("pub struct OrderApiEntity {"));
    assert!(content.contains("pub id: String,"));
    assert!(content.contains("pub name: String,"));
    assert!(content.contains("pub description: Option<String>,"));
    assert!(content.contains("pub status: String,"));
    assert!(content.contains("pub created_at: String,"));
    assert!(content.contains("pub updated_at: String,"));
}

#[test]
fn test_rust_server_rest_domain_repository() {
    let (tmp, _) = render_server("rust", "rest", false, "", false, false);
    let content = read_output(&tmp, "src/domain/repository.rs");

    assert!(content.contains("use async_trait::async_trait;"));
    assert!(content.contains("use super::model::OrderApiEntity;"));
    assert!(content.contains("#[cfg_attr(test, mockall::automock)]"));
    assert!(content.contains("#[async_trait]"));
    assert!(content.contains("pub trait OrderApiRepository: Send + Sync {"));
    assert!(content.contains("async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<OrderApiEntity>>;"));
    assert!(content.contains("async fn find_all(&self) -> anyhow::Result<Vec<OrderApiEntity>>;"));
    assert!(content.contains("async fn create(&self, entity: &OrderApiEntity) -> anyhow::Result<()>;"));
    assert!(content.contains("async fn update(&self, entity: &OrderApiEntity) -> anyhow::Result<()>;"));
    assert!(content.contains("async fn delete(&self, id: &str) -> anyhow::Result<()>;"));
}

#[test]
fn test_rust_server_rest_usecase_service() {
    let (tmp, _) = render_server("rust", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "src/usecase/service.rs");

    assert!(content.contains("use std::sync::Arc;"));
    assert!(content.contains("use crate::domain::model::OrderApiEntity;"));
    assert!(content.contains("use crate::domain::repository::OrderApiRepository;"));
    assert!(content.contains("pub struct OrderApiUseCase {"));
    assert!(content.contains("repo: Arc<dyn OrderApiRepository>,"));
    assert!(content.contains("pub fn new("));
    assert!(content.contains("repo: impl OrderApiRepository + 'static,"));
    assert!(content.contains("pub async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<OrderApiEntity>>"));
    assert!(content.contains("self.repo.find_by_id(id).await"));
    assert!(content.contains("pub async fn get_all(&self) -> anyhow::Result<Vec<OrderApiEntity>>"));
    assert!(content.contains("self.repo.find_all().await"));
}

#[test]
fn test_rust_server_rest_handler() {
    let (tmp, _) = render_server("rust", "rest", false, "", false, false);
    let content = read_output(&tmp, "src/adapter/handler/rest.rs");

    assert!(content.contains("use axum::{"));
    assert!(content.contains("use crate::usecase::OrderApiUseCase;"));
    assert!(content.contains("pub struct AppHandler {"));
    assert!(content.contains("uc: Arc<OrderApiUseCase>,"));
    assert!(content.contains("pub fn new(uc: OrderApiUseCase) -> Self"));
    assert!(content.contains("pub fn routes(&self) -> Router"));
    assert!(content.contains("\"/api/v1/order-api\""));
    assert!(content.contains("\"/api/v1/order-api/:id\""));
    assert!(content.contains("struct ErrorResponse {"));
    assert!(content.contains("async fn list("));
    assert!(content.contains("async fn get_by_id("));
    assert!(content.contains("async fn create("));
    assert!(content.contains("struct CreateRequest {"));
}

#[test]
fn test_rust_server_rest_handler_mod() {
    let (tmp, _) = render_server("rust", "rest", false, "", false, false);
    let content = read_output(&tmp, "src/adapter/handler/mod.rs");

    assert!(content.contains("mod rest;"));
    assert!(content.contains("pub use rest::AppHandler;"));
}

#[test]
fn test_rust_server_rest_infra_mod() {
    let (tmp, _) = render_server("rust", "rest", true, "postgresql", true, false);
    let content = read_output(&tmp, "src/infra/mod.rs");

    assert!(content.contains("pub mod config;"));
    assert!(content.contains("pub mod persistence;"));
    assert!(content.contains("pub mod messaging;"));
}

#[test]
fn test_rust_server_rest_persistence() {
    let (tmp, _) = render_server("rust", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "src/infra/persistence.rs");

    assert!(content.contains("use async_trait::async_trait;"));
    assert!(content.contains("use crate::domain::model::OrderApiEntity;"));
    assert!(content.contains("use crate::domain::repository::OrderApiRepository;"));
    assert!(content.contains("use crate::infra::config::DatabaseConfig;"));
    assert!(content.contains("pub async fn create_pool(cfg: &DatabaseConfig) -> anyhow::Result<DbPool>"));
    assert!(content.contains("pub struct Repository {"));
    assert!(content.contains("impl OrderApiRepository for Repository {"));
    assert!(content.contains("async fn find_by_id(&self, id: &str)"));
    assert!(content.contains("async fn find_all(&self)"));
    assert!(content.contains("async fn create(&self, entity: &OrderApiEntity)"));
    assert!(content.contains("async fn update(&self, entity: &OrderApiEntity)"));
    assert!(content.contains("async fn delete(&self, id: &str)"));
}

#[test]
fn test_rust_server_rest_messaging() {
    let (tmp, _) = render_server("rust", "rest", false, "", true, false);
    let content = read_output(&tmp, "src/infra/messaging.rs");

    assert!(content.contains("use rdkafka::config::ClientConfig;"));
    assert!(content.contains("use rdkafka::producer::{FutureProducer, FutureRecord};"));
    assert!(content.contains("use crate::infra::config::KafkaConfig;"));
    assert!(content.contains("pub struct Producer {"));
    assert!(content.contains("pub fn new(cfg: &KafkaConfig) -> Self"));
    assert!(content.contains("pub async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()>"));
    assert!(content.contains("pub struct KafkaConsumer {"));
    assert!(content.contains("pub fn new(cfg: &KafkaConfig, group_id: &str, topics: &[&str]) -> Self"));
    // 命名規則
    assert!(content.contains("k1s0.{tier}.{domain}.{event-type}.{version}"));
    assert!(content.contains("{service-name}.{purpose}"));
}

#[test]
fn test_rust_server_rest_config_rs() {
    let (tmp, _) = render_server("rust", "rest", true, "postgresql", true, true);
    let content = read_output(&tmp, "src/infra/config.rs");

    assert!(content.contains("use serde::Deserialize;"));
    assert!(content.contains("pub struct Config {"));
    assert!(content.contains("pub app: AppConfig,"));
    assert!(content.contains("pub server: ServerConfig,"));
    assert!(content.contains("pub database: Option<DatabaseConfig>,"));
    assert!(content.contains("pub kafka: Option<KafkaConfig>,"));
    assert!(content.contains("pub redis: Option<RedisConfig>,"));
    assert!(content.contains("pub observability: ObservabilityConfig,"));
    assert!(content.contains("pub struct DatabaseConfig {"));
    assert!(content.contains("pub ssl_mode: String,")); // postgresql
    assert!(content.contains("pub fn connection_string(&self) -> String"));
    assert!(content.contains("postgres://"));
    assert!(content.contains("pub fn load(path: &str) -> anyhow::Result<Self>"));
}

#[test]
fn test_rust_server_rest_dockerfile() {
    let (tmp, _) = render_server("rust", "rest", false, "", false, false);
    let content = read_output(&tmp, "Dockerfile");

    assert!(content.contains("FROM rust:1.82-bookworm AS builder"));
    assert!(content.contains("cargo build --release"));
    assert!(content.contains("FROM gcr.io/distroless/cc-debian12"));
    assert!(content.contains("/app/target/release/order-api /app/server"));
    assert!(content.contains("EXPOSE 8080"));
    assert!(content.contains("USER nonroot:nonroot"));
    assert!(content.contains("ENTRYPOINT [\"/app/server\"]"));
}

// =========================================================================
// Rust サーバー: gRPC
// =========================================================================

#[test]
fn test_rust_server_grpc_file_list() {
    let (_, names) = render_server("rust", "grpc", false, "", false, false);

    assert!(names.iter().any(|n| n.contains("grpc.rs")));
    assert!(names.iter().any(|n| n.contains("buf.yaml")), "buf.yaml missing for Rust gRPC");
    assert!(names.iter().any(|n| n.contains("build.rs")), "build.rs missing for Rust gRPC");
    assert!(names.iter().any(|n| n.contains("integration_test.rs")), "integration_test.rs missing");
    assert!(!names.iter().any(|n| n.contains("rest.rs")));
    assert!(!names.iter().any(|n| n.contains("graphql.rs")));
    assert!(!names.iter().any(|n| n.contains("schema.graphql")));
    assert!(!names.iter().any(|n| n.contains("gqlgen.yml")));
}

#[test]
fn test_rust_server_grpc_handler() {
    let (tmp, _) = render_server("rust", "grpc", false, "", false, false);
    let content = read_output(&tmp, "src/adapter/handler/grpc.rs");

    assert!(content.contains("use tonic::{Request, Response, Status};"));
    assert!(content.contains("use crate::usecase::OrderApiUseCase;"));
    assert!(content.contains("tonic::include_proto!(\"order_api.v1\");"));
    assert!(content.contains("pub struct OrderApiGrpcService {"));
    assert!(content.contains("impl OrderApiService for OrderApiGrpcService {"));
    assert!(content.contains("async fn get_order_api("));
    assert!(content.contains("async fn list_order_api("));
    assert!(content.contains("async fn create_order_api("));
}

#[test]
fn test_rust_server_grpc_handler_mod() {
    let (tmp, _) = render_server("rust", "grpc", false, "", false, false);
    let content = read_output(&tmp, "src/adapter/handler/mod.rs");

    assert!(content.contains("mod grpc;"));
    assert!(content.contains("pub use grpc::OrderApiGrpcService as AppHandler;"));
}

#[test]
fn test_rust_server_grpc_cargo_toml() {
    let (tmp, _) = render_server("rust", "grpc", false, "", false, false);
    let content = read_output(&tmp, "Cargo.toml");

    assert!(content.contains("tonic = \"0.12\""));
    assert!(content.contains("prost = \"0.13\""));
    assert!(content.contains("[build-dependencies]"));
    assert!(content.contains("tonic-build = \"0.12\""));
    assert!(!content.contains("utoipa"));
    assert!(!content.contains("async-graphql"));
}

// =========================================================================
// Rust サーバー: GraphQL
// =========================================================================

#[test]
fn test_rust_server_graphql_file_list() {
    let (_, names) = render_server("rust", "graphql", false, "", false, false);

    assert!(names.iter().any(|n| n.contains("graphql.rs")));
    assert!(names.iter().any(|n| n.contains("integration_test.rs")), "integration_test.rs missing");
    assert!(!names.iter().any(|n| n.contains("rest.rs")));
    assert!(!names.iter().any(|n| n.contains("grpc.rs")));
    assert!(!names.iter().any(|n| n.contains("buf.yaml")));
    assert!(!names.iter().any(|n| n.contains("build.rs")));
}

#[test]
fn test_rust_server_graphql_handler() {
    let (tmp, _) = render_server("rust", "graphql", false, "", false, false);
    let content = read_output(&tmp, "src/adapter/handler/graphql.rs");

    assert!(content.contains("use async_graphql::{Context, Object, Schema, EmptyMutation, EmptySubscription};"));
    assert!(content.contains("use crate::domain::model::OrderApiEntity;"));
    assert!(content.contains("use crate::usecase::OrderApiUseCase;"));
    assert!(content.contains("pub struct QueryRoot;"));
    assert!(content.contains("#[Object]"));
    assert!(content.contains("async fn order_api("));
    assert!(content.contains("async fn order_api_list("));
    assert!(content.contains("pub type OrderApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;"));
    assert!(content.contains("pub fn build_schema(uc: OrderApiUseCase) -> OrderApiSchema"));
}

#[test]
fn test_rust_server_graphql_handler_mod() {
    let (tmp, _) = render_server("rust", "graphql", false, "", false, false);
    let content = read_output(&tmp, "src/adapter/handler/mod.rs");

    assert!(content.contains("mod graphql;"));
    assert!(content.contains("pub use graphql::{build_schema, OrderApiSchema, QueryRoot};"));
}

#[test]
fn test_rust_server_graphql_cargo_toml() {
    let (tmp, _) = render_server("rust", "graphql", false, "", false, false);
    let content = read_output(&tmp, "Cargo.toml");

    assert!(content.contains("async-graphql = \"7\""));
    assert!(content.contains("async-graphql-axum = \"7\""));
    assert!(!content.contains("utoipa"));
    assert!(!content.contains("tonic"));
}

// =========================================================================
// Rust サーバー: DB なし
// =========================================================================

#[test]
fn test_rust_server_no_database() {
    let (tmp, names) = render_server("rust", "rest", false, "", false, false);

    // DB 関連ファイルは除外
    assert!(!names.iter().any(|n| n.contains("persistence")));

    // model に sqlx::FromRow がない
    let model = read_output(&tmp, "src/domain/model.rs");
    assert!(!model.contains("sqlx::FromRow"));

    // usecase に repo がない
    let usecase = read_output(&tmp, "src/usecase/service.rs");
    assert!(!usecase.contains("repo: Arc<dyn"));
    assert!(usecase.contains("// TODO:"));

    // infra/mod.rs に persistence がない
    let infra_mod = read_output(&tmp, "src/infra/mod.rs");
    assert!(!infra_mod.contains("persistence"));

    // main.rs に persistence がない
    let main = read_output(&tmp, "src/main.rs");
    assert!(!main.contains("persistence"));

    // config に database がない
    let config = read_output(&tmp, "src/infra/config.rs");
    assert!(!config.contains("DatabaseConfig"));

    // config.yaml に database がない
    let config_yaml = read_output(&tmp, "config/config.yaml");
    assert!(!config_yaml.contains("database:"));

    // Cargo.toml に sqlx がない
    let cargo = read_output(&tmp, "Cargo.toml");
    assert!(!cargo.contains("sqlx"));
}

// =========================================================================
// Rust サーバー: MySQL
// =========================================================================

#[test]
fn test_rust_server_mysql_config() {
    let (tmp, _) = render_server("rust", "rest", true, "mysql", false, false);

    let cargo = read_output(&tmp, "Cargo.toml");
    assert!(cargo.contains("features = [\"runtime-tokio-rustls\", \"mysql\"]"));

    let config_rs = read_output(&tmp, "src/infra/config.rs");
    assert!(!config_rs.contains("pub ssl_mode: String,")); // mysql has no ssl_mode
    assert!(config_rs.contains("mysql://"));

    let config_yaml = read_output(&tmp, "config/config.yaml");
    assert!(config_yaml.contains("port: 3306"));
}

// =========================================================================
// Tera 変数置換の正確性テスト
// =========================================================================

#[test]
fn test_tera_variable_substitution_consistency() {
    // service_name = "user-auth" でケース変換が正しいか
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("user-auth", "service", "go", "server")
        .api_style("rest")
        .with_database("postgresql")
        .build();

    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    engine.render_to_dir(&ctx, &output_dir).unwrap();

    // entity: PascalCase
    let entity = fs::read_to_string(output_dir.join("internal/domain/model/entity.go")).unwrap();
    assert!(entity.contains("type UserAuthEntity struct {"));

    // repository: PascalCase
    let repo = fs::read_to_string(output_dir.join("internal/domain/repository/repository.go")).unwrap();
    assert!(repo.contains("type UserAuthRepository interface {"));

    // usecase: PascalCase
    let uc = fs::read_to_string(output_dir.join("internal/usecase/usecase.go")).unwrap();
    assert!(uc.contains("type UserAuthUseCase struct {"));
    assert!(uc.contains("func NewUserAuthUseCase("));

    // persistence: camelCase
    let persistence = fs::read_to_string(output_dir.join("internal/infra/persistence/repository.go")).unwrap();
    assert!(persistence.contains("type userAuthRepository struct {"));

    // REST handler: PascalCase in type, kebab-case in route
    let handler = fs::read_to_string(output_dir.join("internal/adapter/handler/rest_handler.go")).unwrap();
    assert!(handler.contains("uc *usecase.UserAuthUseCase"));
    assert!(handler.contains("v1.GET(\"/user-auth\""));

    // go.mod: service_name in path
    let go_mod = fs::read_to_string(output_dir.join("go.mod")).unwrap();
    assert!(go_mod.contains("module github.com/org/k1s0/regions/service/user-auth/server/go"));

    // config.yaml: kebab-case, snake_case
    let config_yaml = fs::read_to_string(output_dir.join("config/config.yaml")).unwrap();
    assert!(config_yaml.contains("name: \"user-auth\""));
    assert!(config_yaml.contains("user: \"app\""));
    assert!(config_yaml.contains("name: \"user_auth_db\""));
}

#[test]
fn test_tera_variable_substitution_rust() {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("user-auth", "service", "rust", "server")
        .api_style("rest")
        .with_database("postgresql")
        .build();

    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    engine.render_to_dir(&ctx, &output_dir).unwrap();

    // Cargo.toml: rust_crate
    let cargo = fs::read_to_string(output_dir.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("name = \"user-auth\""));

    // model: PascalCase
    let model = fs::read_to_string(output_dir.join("src/domain/model.rs")).unwrap();
    assert!(model.contains("pub struct UserAuthEntity {"));

    // repository: PascalCase
    let repo = fs::read_to_string(output_dir.join("src/domain/repository.rs")).unwrap();
    assert!(repo.contains("pub trait UserAuthRepository: Send + Sync {"));

    // usecase
    let uc = fs::read_to_string(output_dir.join("src/usecase/service.rs")).unwrap();
    assert!(uc.contains("pub struct UserAuthUseCase {"));
    assert!(uc.contains("repo: Arc<dyn UserAuthRepository>,"));

    // usecase/mod.rs
    let uc_mod = fs::read_to_string(output_dir.join("src/usecase/mod.rs")).unwrap();
    assert!(uc_mod.contains("pub use service::UserAuthUseCase;"));

    // REST handler
    let handler = fs::read_to_string(output_dir.join("src/adapter/handler/rest.rs")).unwrap();
    assert!(handler.contains("use crate::usecase::UserAuthUseCase;"));
    assert!(handler.contains("\"/api/v1/user-auth\""));

    // Dockerfile
    let dockerfile = fs::read_to_string(output_dir.join("Dockerfile")).unwrap();
    assert!(dockerfile.contains("/app/target/release/user-auth /app/server"));
}

// =========================================================================
// パイプライン関連テスト
// =========================================================================

#[test]
fn test_go_server_graphql_file_list_pipeline() {
    let (_, names) = render_server("go", "graphql", false, "", false, false);

    assert!(names.iter().any(|n| n.contains("graphql_resolver")), "graphql_resolver missing");
    assert!(names.iter().any(|n| n.contains("schema.graphql")), "schema.graphql missing");
    assert!(names.iter().any(|n| n.contains("gqlgen.yml")), "gqlgen.yml missing");
    assert!(!names.iter().any(|n| n.contains("rest_handler")), "rest_handler should not exist for GraphQL");
    assert!(!names.iter().any(|n| n.contains("service.proto")), "service.proto should not exist for GraphQL");
}

#[test]
fn test_go_server_multi_api_file_list() {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("order-api", "service", "go", "server")
        .api_styles(vec!["rest".to_string(), "grpc".to_string()])
        .with_database("postgresql")
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

    // REST + gRPC の両方が含まれる
    assert!(names.iter().any(|n| n.contains("rest_handler")), "rest_handler missing for multi-api");
    assert!(names.iter().any(|n| n.contains("grpc_handler")), "grpc_handler missing for multi-api");
    assert!(names.iter().any(|n| n.contains("openapi")), "openapi missing for multi-api");
    assert!(names.iter().any(|n| n.contains("proto/")), "proto missing for multi-api");
}

// =========================================================================
// サーバー: oapi-codegen.yaml, gqlgen.yml, schema.graphql, README, build.rs 内容検証
// =========================================================================

#[test]
fn test_go_server_rest_oapi_codegen_yaml() {
    let (tmp, names) = render_server("go", "rest", false, "", false, false);

    assert!(
        names.iter().any(|n| n == "oapi-codegen.yaml"),
        "oapi-codegen.yaml missing for REST"
    );

    let content = read_output(&tmp, "oapi-codegen.yaml");
    assert!(content.contains("package: openapi"));
    assert!(content.contains("gin-server: true"));
    assert!(content.contains("models: true"));
    assert!(content.contains("embedded-spec: true"));
    assert!(content.contains("internal/adapter/handler/openapi_gen.go"));
}

#[test]
fn test_go_server_graphql_schema_graphql() {
    let (tmp, _) = render_server("go", "graphql", false, "", false, false);
    let content = read_output(&tmp, "api/graphql/schema.graphql");

    assert!(content.contains("type Query"));
    assert!(content.contains("OrderApi"));
    assert!(content.contains("CreateOrderApiInput"));
}

#[test]
fn test_go_server_graphql_gqlgen_yml() {
    let (tmp, _) = render_server("go", "graphql", false, "", false, false);
    let content = read_output(&tmp, "gqlgen.yml");

    assert!(content.contains("api/graphql/*.graphql"));
    assert!(content.contains("package: graphql"));
    assert!(content.contains("layout: follow-schema"));
}

#[test]
fn test_go_server_rest_readme() {
    let (tmp, _) = render_server("go", "rest", false, "", false, false);
    let content = read_output(&tmp, "README.md");

    assert!(content.contains("# order-api"));
    assert!(content.contains("go mod tidy"));
    assert!(content.contains("go run"));
    assert!(content.contains("go test"));
    assert!(content.contains("cmd/"));
    assert!(content.contains("internal/"));
}

#[test]
fn test_rust_server_rest_readme() {
    let (tmp, _) = render_server("rust", "rest", false, "", false, false);
    let content = read_output(&tmp, "README.md");

    assert!(content.contains("# order-api"));
    assert!(content.contains("cargo build"));
    assert!(content.contains("cargo run"));
    assert!(content.contains("cargo test"));
    assert!(content.contains("src/"));
    assert!(content.contains("Cargo.toml"));
}

#[test]
fn test_rust_server_grpc_build_rs() {
    let (tmp, _) = render_server("rust", "grpc", false, "", false, false);
    let content = read_output(&tmp, "build.rs");

    assert!(content.contains("tonic_build"));
    assert!(content.contains("build_server(true)"));
    assert!(content.contains("compile_protos"));
}

#[test]
fn test_go_server_grpc_buf_yaml() {
    let (tmp, _) = render_server("go", "grpc", false, "", false, false);
    let content = read_output(&tmp, "buf.yaml");

    assert!(content.contains("version: v2"));
    assert!(content.contains("STANDARD"));
    assert!(content.contains("FILE"));
}

#[test]
fn test_go_server_grpc_buf_gen_yaml() {
    let (tmp, _) = render_server("go", "grpc", false, "", false, false);
    let content = read_output(&tmp, "buf.gen.yaml");

    assert!(content.contains("buf.build/protocolbuffers/go"));
    assert!(content.contains("buf.build/grpc/go"));
    assert!(content.contains("api/proto/gen"));
}

// =========================================================================
// サーバー: テストファイル内容検証
// =========================================================================

#[test]
fn test_go_server_rest_usecase_test_content() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/usecase/usecase_test.go");

    assert!(content.contains("package usecase"));
    assert!(content.contains("testify/assert"));
    assert!(content.contains("testify/require"));
    assert!(content.contains("gomock.NewController(t)"));
    assert!(content.contains("TestGetByID"));
    assert!(content.contains("TestGetAll"));
    assert!(content.contains("TestCreate"));
}

#[test]
fn test_go_server_rest_handler_test_content() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/adapter/handler/handler_test.go");

    assert!(content.contains("package handler"));
    assert!(content.contains("httptest"));
    assert!(content.contains("TestList"));
    assert!(content.contains("TestGetByID"));
    assert!(content.contains("TestCreate"));
}

#[test]
fn test_go_server_grpc_handler_test_content() {
    let (tmp, _) = render_server("go", "grpc", false, "", false, false);
    let content = read_output(&tmp, "internal/adapter/handler/handler_test.go");

    assert!(content.contains("package handler"));
    assert!(content.contains("bufconn"));
    assert!(content.contains("TestGRPCPlaceholder"));
}

#[test]
fn test_go_server_graphql_handler_test_content() {
    let (tmp, _) = render_server("go", "graphql", false, "", false, false);
    let content = read_output(&tmp, "internal/adapter/handler/handler_test.go");

    assert!(content.contains("package handler"));
    assert!(content.contains("TestGraphQLResolverCreation"));
    assert!(content.contains("NewResolver"));
}

#[test]
fn test_go_server_rest_repository_test_content() {
    let (tmp, _) = render_server("go", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "internal/infra/persistence/repository_test.go");

    assert!(content.contains("package persistence"));
    assert!(content.contains("testcontainers"));
    assert!(content.contains("testing.Short()"));
    assert!(content.contains("postgres.Run"));
}

#[test]
fn test_rust_server_rest_integration_test_content() {
    let (tmp, _) = render_server("rust", "rest", true, "postgresql", false, false);
    let content = read_output(&tmp, "tests/integration_test.rs");

    assert!(content.contains("use axum::body::Body"));
    assert!(content.contains("StatusCode::OK"));
    assert!(content.contains("test_list_returns_ok"));
    assert!(content.contains("test_get_by_id_not_found"));
}

#[test]
fn test_rust_server_graphql_integration_test_content() {
    let (tmp, _) = render_server("rust", "graphql", false, "", false, false);
    let content = read_output(&tmp, "tests/integration_test.rs");

    assert!(content.contains("build_schema"));
    assert!(content.contains("test_graphql_schema_creation"));
}

#[test]
fn test_rust_server_rest_no_tera_syntax_in_output() {
    let (tmp, names) = render_server("rust", "rest", true, "postgresql", false, false);

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{{"), "Tera syntax {{{{ found in {}", name);
        assert!(!content.contains("{%"), "Tera syntax {{%% found in {}", name);
        assert!(!content.contains("{#"), "Tera comment {{# found in {}", name);
    }
}

// =========================================================================
// Go サーバー: Redis キャッシュクライアント
// =========================================================================

#[test]
fn test_go_server_rest_redis_cache_client() {
    let (tmp, names) = render_server("go", "rest", false, "", false, true);

    assert!(
        names.iter().any(|n| n.contains("cache/redis.go")),
        "cache/redis.go missing when has_redis=true"
    );

    let content = read_output(&tmp, "internal/infra/cache/redis.go");
    assert!(content.contains("package cache"));
    assert!(content.contains("type RedisClient struct {"));
    assert!(content.contains("func NewRedisClient(cfg config.RedisConfig) *RedisClient"));
    assert!(content.contains("func (r *RedisClient) Get(ctx context.Context, key string) (string, error)"));
    assert!(content.contains("func (r *RedisClient) Set(ctx context.Context, key string, value interface{}, ttl time.Duration) error"));
    assert!(content.contains("func (r *RedisClient) Delete(ctx context.Context, key string) error"));
    assert!(content.contains("func (r *RedisClient) Close() error"));
}

#[test]
fn test_go_server_rest_no_redis_cache_client() {
    let (_, names) = render_server("go", "rest", false, "", false, false);

    assert!(
        !names.iter().any(|n| n.contains("cache/redis.go")),
        "cache/redis.go should not exist when has_redis=false"
    );
}

// =========================================================================
// Rust サーバー: Redis キャッシュクライアント
// =========================================================================

#[test]
fn test_rust_server_rest_redis_client() {
    let (tmp, names) = render_server("rust", "rest", false, "", false, true);

    assert!(
        names.iter().any(|n| n.contains("redis_client.rs")),
        "redis_client.rs missing when has_redis=true"
    );

    let content = read_output(&tmp, "src/infra/redis_client.rs");
    assert!(content.contains("pub struct RedisClient {"));
    assert!(content.contains("pub fn new(cfg: &RedisConfig) -> Result<Self>"));
    assert!(content.contains("pub async fn get(&self, key: &str) -> Result<Option<String>>"));
    assert!(content.contains("pub async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<()>"));
    assert!(content.contains("pub async fn delete(&self, key: &str) -> Result<()>"));
}

#[test]
fn test_rust_server_rest_no_redis_client() {
    let (_, names) = render_server("rust", "rest", false, "", false, false);

    assert!(
        !names.iter().any(|n| n.contains("redis_client.rs")),
        "redis_client.rs should not exist when has_redis=false"
    );
}
