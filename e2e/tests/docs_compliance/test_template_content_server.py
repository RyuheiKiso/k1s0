"""テンプレート仕様-サーバー.md の内容準拠テスト。

CLI/templates/server/ のテンプレートファイルの内容が
仕様ドキュメントのコードブロックと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"
SERVER_GO = TEMPLATES / "server" / "go"
SERVER_RUST = TEMPLATES / "server" / "rust"


class TestGoModContent:
    """テンプレート仕様-サーバー.md: go.mod.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_GO / "go.mod.tera").read_text(encoding="utf-8")

    def test_module_variable(self) -> None:
        assert "{{ go_module }}" in self.content

    def test_go_version(self) -> None:
        assert "go 1.23" in self.content

    def test_gin_dependency(self) -> None:
        assert "github.com/gin-gonic/gin" in self.content

    def test_otel_dependency(self) -> None:
        assert "go.opentelemetry.io/otel" in self.content

    def test_validator_dependency(self) -> None:
        assert "github.com/go-playground/validator/v10" in self.content

    def test_yaml_dependency(self) -> None:
        assert "gopkg.in/yaml.v3" in self.content

    @pytest.mark.parametrize(
        "condition,dep",
        [
            ('api_style == "rest"', "oapi-codegen"),
            ('api_style == "grpc"', "google.golang.org/grpc"),
            ('api_style == "graphql"', "gqlgen"),
            ("has_database", "github.com/jmoiron/sqlx"),
            ('database_type == "postgresql"', "github.com/lib/pq"),
            ('database_type == "mysql"', "go-sql-driver/mysql"),
            ('database_type == "sqlite"', "go-sqlite3"),
            ("has_kafka", "kafka-go"),
            ("has_redis", "go-redis"),
        ],
    )
    def test_conditional_dependency(self, condition: str, dep: str) -> None:
        """テンプレート仕様-サーバー.md: 条件付き依存が定義されている。"""
        assert condition in self.content
        assert dep in self.content


class TestGoMainContent:
    """テンプレート仕様-サーバー.md: cmd/main.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_GO / "cmd" / "main.go.tera").read_text(encoding="utf-8")

    def test_package_main(self) -> None:
        assert "package main" in self.content

    def test_gin_import(self) -> None:
        assert '"github.com/gin-gonic/gin"' in self.content

    def test_otelgin_middleware(self) -> None:
        assert "otelgin.Middleware" in self.content

    def test_config_load(self) -> None:
        assert 'config.Load("config/config.yaml")' in self.content

    def test_slog_json_handler(self) -> None:
        assert "slog.NewJSONHandler" in self.content

    def test_healthz_endpoint(self) -> None:
        assert '"/healthz"' in self.content

    def test_readyz_endpoint(self) -> None:
        assert '"/readyz"' in self.content

    def test_graceful_shutdown(self) -> None:
        assert "srv.Shutdown" in self.content
        assert "syscall.SIGTERM" in self.content

    def test_database_conditional(self) -> None:
        assert "{% if has_database %}" in self.content
        assert "persistence.NewDB" in self.content

    def test_kafka_conditional(self) -> None:
        assert "{% if has_kafka %}" in self.content
        assert "messaging.NewProducer" in self.content

    def test_di_pattern(self) -> None:
        assert "usecase.New{{ service_name_pascal }}UseCase" in self.content
        assert "handler.NewHandler" in self.content


class TestGoEntityContent:
    """テンプレート仕様-サーバー.md: entity.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "domain" / "model" / "entity.go.tera"
        ).read_text(encoding="utf-8")

    def test_package_model(self) -> None:
        assert "package model" in self.content

    def test_entity_struct(self) -> None:
        assert "{{ service_name_pascal }}Entity struct" in self.content

    def test_fields(self) -> None:
        assert "ID" in self.content
        assert "Name" in self.content
        assert "CreatedAt" in self.content
        assert "UpdatedAt" in self.content

    def test_json_tags(self) -> None:
        assert '`json:"id"' in self.content

    def test_db_tags(self) -> None:
        assert 'db:"id"' in self.content

    def test_validate_tag(self) -> None:
        assert 'validate:"required,max=255"' in self.content


class TestGoRepositoryContent:
    """テンプレート仕様-サーバー.md: repository.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "domain" / "repository" / "repository.go.tera"
        ).read_text(encoding="utf-8")

    def test_package_repository(self) -> None:
        assert "package repository" in self.content

    def test_mockgen_directive(self) -> None:
        assert "//go:generate mockgen" in self.content

    def test_interface_methods(self) -> None:
        assert "FindByID" in self.content
        assert "FindAll" in self.content
        assert "Create" in self.content
        assert "Update" in self.content
        assert "Delete" in self.content


class TestGoUsecaseContent:
    """テンプレート仕様-サーバー.md: usecase.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "usecase" / "usecase.go.tera"
        ).read_text(encoding="utf-8")

    def test_package_usecase(self) -> None:
        assert "package usecase" in self.content

    def test_di_struct(self) -> None:
        assert "{{ service_name_pascal }}UseCase struct" in self.content

    def test_constructor(self) -> None:
        assert "New{{ service_name_pascal }}UseCase" in self.content

    def test_get_by_id(self) -> None:
        assert "GetByID" in self.content

    def test_get_all(self) -> None:
        assert "GetAll" in self.content


class TestGoHandlerRestContent:
    """テンプレート仕様-サーバー.md: rest_handler.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "adapter" / "handler" / "rest_handler.go.tera"
        ).read_text(encoding="utf-8")

    def test_rest_conditional(self) -> None:
        assert '{% if api_style == "rest" %}' in self.content

    def test_gin_handler(self) -> None:
        assert '"github.com/gin-gonic/gin"' in self.content

    def test_register_routes(self) -> None:
        assert "RegisterRoutes" in self.content

    def test_error_response(self) -> None:
        """テンプレート仕様-サーバー.md: API設計.md D-007 準拠のエラーレスポンス。"""
        assert "ErrorResponse" in self.content
        assert "INTERNAL_ERROR" in self.content
        assert "NOT_FOUND" in self.content
        assert "VALIDATION_ERROR" in self.content

    def test_validator(self) -> None:
        assert "validator.New()" in self.content


class TestGoHandlerGrpcContent:
    """テンプレート仕様-サーバー.md: grpc_handler.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "adapter" / "handler" / "grpc_handler.go.tera"
        ).read_text(encoding="utf-8")

    def test_grpc_conditional(self) -> None:
        assert '{% if api_style == "grpc" %}' in self.content

    def test_grpc_imports(self) -> None:
        assert "google.golang.org/grpc/codes" in self.content
        assert "google.golang.org/grpc/status" in self.content

    def test_service_server(self) -> None:
        assert "Unimplemented{{ service_name_pascal }}ServiceServer" in self.content


class TestGoHandlerGraphqlContent:
    """テンプレート仕様-サーバー.md: graphql_resolver.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "adapter" / "handler" / "graphql_resolver.go.tera"
        ).read_text(encoding="utf-8")

    def test_graphql_conditional(self) -> None:
        assert '{% if api_style == "graphql" %}' in self.content

    def test_resolver_struct(self) -> None:
        assert "Resolver struct" in self.content

    def test_query_resolver(self) -> None:
        assert "queryResolver" in self.content


class TestGoDbContent:
    """テンプレート仕様-サーバー.md: db.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "infra" / "persistence" / "db.go.tera"
        ).read_text(encoding="utf-8")

    def test_database_conditional(self) -> None:
        assert "{% if has_database %}" in self.content

    def test_sqlx_import(self) -> None:
        assert '"github.com/jmoiron/sqlx"' in self.content

    def test_db_drivers(self) -> None:
        assert "github.com/lib/pq" in self.content
        assert "go-sql-driver/mysql" in self.content
        assert "go-sqlite3" in self.content

    def test_connection_pool(self) -> None:
        assert "SetMaxOpenConns" in self.content
        assert "SetMaxIdleConns" in self.content
        assert "SetConnMaxLifetime" in self.content


class TestGoKafkaContent:
    """テンプレート仕様-サーバー.md: kafka.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "infra" / "messaging" / "kafka.go.tera"
        ).read_text(encoding="utf-8")

    def test_kafka_conditional(self) -> None:
        assert "{% if has_kafka %}" in self.content

    def test_producer(self) -> None:
        assert "Producer struct" in self.content
        assert "NewProducer" in self.content
        assert "Publish" in self.content

    def test_consumer(self) -> None:
        assert "Consumer struct" in self.content
        assert "NewConsumer" in self.content
        assert "Consume" in self.content

    def test_topic_naming_comment(self) -> None:
        """テンプレート仕様-サーバー.md: トピック命名規則コメント。"""
        assert "k1s0.{tier}.{domain}.{event-type}.{version}" in self.content


class TestGoConfigContent:
    """テンプレート仕様-サーバー.md: config.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            SERVER_GO / "internal" / "infra" / "config" / "config.go.tera"
        ).read_text(encoding="utf-8")

    def test_config_struct(self) -> None:
        assert "Config struct" in self.content

    def test_app_config(self) -> None:
        assert "AppConfig" in self.content

    def test_server_config(self) -> None:
        assert "ServerConfig" in self.content

    def test_observability_config(self) -> None:
        assert "ObservabilityConfig" in self.content

    def test_load_function(self) -> None:
        assert "func Load" in self.content
        assert "yaml.Unmarshal" in self.content


class TestGoConfigYamlContent:
    """テンプレート仕様-サーバー.md: config.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_GO / "config" / "config.yaml.tera").read_text(encoding="utf-8")

    def test_app_section(self) -> None:
        assert "app:" in self.content
        assert "{{ service_name }}" in self.content

    def test_server_section(self) -> None:
        assert "server:" in self.content
        assert "8080" in self.content

    def test_observability_section(self) -> None:
        assert "observability:" in self.content
        assert "trace" in self.content


class TestGoDockerfileContent:
    """テンプレート仕様-サーバー.md: Dockerfile.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_GO / "Dockerfile.tera").read_text(encoding="utf-8")

    def test_multistage_build(self) -> None:
        assert "AS builder" in self.content

    def test_go_base_image(self) -> None:
        assert "golang:1.23" in self.content

    def test_distroless_runtime(self) -> None:
        assert "distroless" in self.content

    def test_nonroot_user(self) -> None:
        assert "nonroot:nonroot" in self.content

    def test_expose(self) -> None:
        assert "EXPOSE 8080" in self.content

    def test_cgo_disabled(self) -> None:
        assert "CGO_ENABLED=0" in self.content


class TestRustCargoTomlContent:
    """テンプレート仕様-サーバー.md: Cargo.toml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_RUST / "Cargo.toml.tera").read_text(encoding="utf-8")

    def test_crate_name(self) -> None:
        assert "{{ rust_crate }}" in self.content

    def test_axum(self) -> None:
        assert 'axum = "0.7"' in self.content

    def test_tokio(self) -> None:
        assert "tokio" in self.content

    def test_serde(self) -> None:
        assert "serde" in self.content

    def test_tracing(self) -> None:
        assert "tracing" in self.content

    def test_otel(self) -> None:
        assert "opentelemetry" in self.content

    def test_thiserror(self) -> None:
        assert 'thiserror = "2"' in self.content

    @pytest.mark.parametrize(
        "condition,dep",
        [
            ('api_style == "rest"', "utoipa"),
            ('api_style == "grpc"', "tonic"),
            ('api_style == "graphql"', "async-graphql"),
            ("has_database", "sqlx"),
            ("has_kafka", "rdkafka"),
            ("has_redis", "redis"),
        ],
    )
    def test_conditional_dependency(self, condition: str, dep: str) -> None:
        assert condition in self.content
        assert dep in self.content

    def test_dev_dependencies(self) -> None:
        assert "mockall" in self.content


class TestRustMainContent:
    """テンプレート仕様-サーバー.md: src/main.rs.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_RUST / "src" / "main.rs.tera").read_text(encoding="utf-8")

    def test_tokio_main(self) -> None:
        assert "#[tokio::main]" in self.content

    def test_axum_router(self) -> None:
        assert "Router::new()" in self.content

    def test_healthz(self) -> None:
        assert '"/healthz"' in self.content

    def test_readyz(self) -> None:
        assert '"/readyz"' in self.content

    def test_graceful_shutdown(self) -> None:
        assert "with_graceful_shutdown" in self.content
        assert "shutdown_signal" in self.content

    def test_tracing_subscriber(self) -> None:
        assert "tracing_subscriber" in self.content


class TestRustDomainContent:
    """テンプレート仕様-サーバー.md: Rust domain テンプレートの内容検証。"""

    def test_model_entity(self) -> None:
        content = (SERVER_RUST / "src" / "domain" / "model.rs.tera").read_text(encoding="utf-8")
        assert "{{ service_name_pascal }}Entity" in content
        assert "Serialize" in content
        assert "Deserialize" in content

    def test_repository_trait(self) -> None:
        content = (SERVER_RUST / "src" / "domain" / "repository.rs.tera").read_text(encoding="utf-8")
        assert "{{ service_name_pascal }}Repository" in content
        assert "async_trait" in content
        assert "mockall" in content

    def test_usecase(self) -> None:
        content = (SERVER_RUST / "src" / "usecase" / "service.rs.tera").read_text(encoding="utf-8")
        assert "{{ service_name_pascal }}UseCase" in content
        assert "Arc" in content


class TestRustHandlerContent:
    """テンプレート仕様-サーバー.md: Rust handler テンプレートの内容検証。"""

    def test_rest_handler(self) -> None:
        content = (SERVER_RUST / "src" / "adapter" / "handler" / "rest.rs.tera").read_text(encoding="utf-8")
        assert '{% if api_style == "rest" %}' in content
        assert "AppHandler" in content
        assert "ErrorResponse" in content

    def test_grpc_handler(self) -> None:
        content = (SERVER_RUST / "src" / "adapter" / "handler" / "grpc.rs.tera").read_text(encoding="utf-8")
        assert '{% if api_style == "grpc" %}' in content
        assert "tonic" in content

    def test_graphql_handler(self) -> None:
        content = (SERVER_RUST / "src" / "adapter" / "handler" / "graphql.rs.tera").read_text(encoding="utf-8")
        assert '{% if api_style == "graphql" %}' in content
        assert "async_graphql" in content
        assert "QueryRoot" in content


class TestRustInfraContent:
    """テンプレート仕様-サーバー.md: Rust infra テンプレートの内容検証。"""

    def test_persistence(self) -> None:
        content = (SERVER_RUST / "src" / "infra" / "persistence.rs.tera").read_text(encoding="utf-8")
        assert "{% if has_database %}" in content
        assert "sqlx" in content
        assert "DbPool" in content

    def test_messaging(self) -> None:
        content = (SERVER_RUST / "src" / "infra" / "messaging.rs.tera").read_text(encoding="utf-8")
        assert "{% if has_kafka %}" in content
        assert "rdkafka" in content
        assert "Producer" in content

    def test_config(self) -> None:
        content = (SERVER_RUST / "src" / "infra" / "config.rs.tera").read_text(encoding="utf-8")
        assert "Config" in content
        assert "AppConfig" in content
        assert "ServerConfig" in content
        assert "ObservabilityConfig" in content
        assert "serde_yaml" in content


class TestRustDockerfileContent:
    """テンプレート仕様-サーバー.md: Rust Dockerfile.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_RUST / "Dockerfile.tera").read_text(encoding="utf-8")

    def test_multistage_build(self) -> None:
        assert "AS builder" in self.content

    def test_rust_base_image(self) -> None:
        assert "rust:1.82" in self.content

    def test_distroless_runtime(self) -> None:
        assert "distroless" in self.content

    def test_nonroot_user(self) -> None:
        assert "nonroot:nonroot" in self.content

    def test_expose(self) -> None:
        assert "EXPOSE 8080" in self.content


class TestGoOpenAPIContent:
    """テンプレート仕様-サーバー.md: openapi.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_GO / "api" / "openapi" / "openapi.yaml.tera").read_text(encoding="utf-8")

    def test_rest_conditional(self) -> None:
        assert '{% if api_style == "rest" %}' in self.content

    def test_openapi_version(self) -> None:
        assert '"3.0.3"' in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name_pascal }}" in self.content

    def test_error_response_schema(self) -> None:
        assert "ErrorResponse" in self.content


class TestGoProtoContent:
    """テンプレート仕様-サーバー.md: service.proto.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (SERVER_GO / "api" / "proto" / "service.proto.tera").read_text(encoding="utf-8")

    def test_grpc_conditional(self) -> None:
        assert '{% if api_style == "grpc" %}' in self.content

    def test_proto3_syntax(self) -> None:
        assert 'syntax = "proto3"' in self.content

    def test_service_definition(self) -> None:
        assert "{{ service_name_pascal }}Service" in self.content

    def test_rpc_methods(self) -> None:
        assert "Get{{ service_name_pascal }}" in self.content
        assert "List{{ service_name_pascal }}" in self.content
        assert "Create{{ service_name_pascal }}" in self.content
