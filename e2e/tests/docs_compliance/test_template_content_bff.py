"""テンプレート仕様 BFF の内容準拠テスト。

CLI/templates/bff/ のテンプレートファイルの存在と内容が
仕様ドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"
BFF_GO = TEMPLATES / "bff" / "go"
BFF_RUST = TEMPLATES / "bff" / "rust"


# ============================================================================
# Go BFF テンプレートファイル存在確認
# ============================================================================


class TestGoBffTemplateFiles:
    """Go BFF テンプレートファイルの存在検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "cmd/main.go.tera",
            "go.mod.tera",
            "internal/handler/graphql_resolver.go.tera",
            "internal/handler/graphql_resolver_test.go.tera",
            "internal/client/upstream.go.tera",
            "api/graphql/schema.graphql.tera",
            "api/graphql/gqlgen.yml.tera",
            "config/config.yaml.tera",
            "Dockerfile.tera",
            "README.md.tera",
        ],
    )
    def test_go_bff_template_exists(self, template: str) -> None:
        path = BFF_GO / template
        assert path.exists(), f"bff/go/{template} が存在しません"


# ============================================================================
# Rust BFF テンプレートファイル存在確認
# ============================================================================


class TestRustBffTemplateFiles:
    """Rust BFF テンプレートファイルの存在検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "src/main.rs.tera",
            "src/handler/graphql.rs.tera",
            "src/handler/mod.rs.tera",
            "src/client/mod.rs.tera",
            "src/client/upstream.rs.tera",
            "tests/integration_test.rs.tera",
            "Cargo.toml.tera",
            "config/config.yaml.tera",
            "Dockerfile.tera",
            "README.md.tera",
        ],
    )
    def test_rust_bff_template_exists(self, template: str) -> None:
        path = BFF_RUST / template
        assert path.exists(), f"bff/rust/{template} が存在しません"


# ============================================================================
# Go BFF テンプレート内容検証
# ============================================================================


class TestGoBffMainContent:
    """Go BFF cmd/main.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_GO / "cmd" / "main.go.tera").read_text(encoding="utf-8")

    def test_package_main(self) -> None:
        assert "package main" in self.content

    def test_go_module_import(self) -> None:
        assert "{{ go_module }}" in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_listen_port(self) -> None:
        assert ":8080" in self.content


class TestGoBffGoModContent:
    """Go BFF go.mod.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_GO / "go.mod.tera").read_text(encoding="utf-8")

    def test_module_variable(self) -> None:
        assert "{{ go_module }}" in self.content

    def test_gqlgen_dependency(self) -> None:
        assert "gqlgen" in self.content

    def test_gqlparser_dependency(self) -> None:
        assert "gqlparser" in self.content


class TestGoBffResolverContent:
    """Go BFF internal/handler/graphql_resolver.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            BFF_GO / "internal" / "handler" / "graphql_resolver.go.tera"
        ).read_text(encoding="utf-8")

    def test_package_handler(self) -> None:
        assert "package handler" in self.content

    def test_resolver_struct(self) -> None:
        assert "Resolver" in self.content

    def test_healthz_endpoint(self) -> None:
        assert "/healthz" in self.content


class TestGoBffSchemaContent:
    """Go BFF api/graphql/schema.graphql.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            BFF_GO / "api" / "graphql" / "schema.graphql.tera"
        ).read_text(encoding="utf-8")

    def test_query_type(self) -> None:
        assert "type Query" in self.content


class TestGoBffGqlgenContent:
    """Go BFF api/graphql/gqlgen.yml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            BFF_GO / "api" / "graphql" / "gqlgen.yml.tera"
        ).read_text(encoding="utf-8")

    def test_schema_path(self) -> None:
        assert "schema" in self.content

    def test_exec_section(self) -> None:
        assert "exec" in self.content

    def test_model_section(self) -> None:
        assert "model" in self.content


class TestGoBffConfigContent:
    """Go BFF config/config.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_GO / "config" / "config.yaml.tera").read_text(
            encoding="utf-8"
        )

    def test_server_section(self) -> None:
        assert "server:" in self.content or "port:" in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_upstream_section(self) -> None:
        assert "upstream:" in self.content


class TestGoBffDockerfileContent:
    """Go BFF Dockerfile.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_GO / "Dockerfile.tera").read_text(encoding="utf-8")

    def test_multistage_build(self) -> None:
        assert "AS builder" in self.content

    def test_go_base_image(self) -> None:
        assert "golang:" in self.content

    def test_distroless_runtime(self) -> None:
        assert "distroless" in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content


class TestGoBffReadmeContent:
    """Go BFF README.md.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_GO / "README.md.tera").read_text(encoding="utf-8")

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_bff_mention(self) -> None:
        assert "BFF" in self.content or "bff" in self.content


# ============================================================================
# Rust BFF テンプレート内容検証
# ============================================================================


class TestRustBffMainContent:
    """Rust BFF src/main.rs.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_RUST / "src" / "main.rs.tera").read_text(encoding="utf-8")

    def test_actix_web(self) -> None:
        assert "actix_web" in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_healthz_route(self) -> None:
        assert "/healthz" in self.content

    def test_listen_port(self) -> None:
        assert "8080" in self.content


class TestRustBffGraphqlHandlerContent:
    """Rust BFF src/handler/graphql.rs.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (
            BFF_RUST / "src" / "handler" / "graphql.rs.tera"
        ).read_text(encoding="utf-8")

    def test_async_graphql(self) -> None:
        assert "async_graphql" in self.content

    def test_build_schema(self) -> None:
        assert "build_schema" in self.content

    def test_query_root(self) -> None:
        assert "QueryRoot" in self.content


class TestRustBffCargoTomlContent:
    """Rust BFF Cargo.toml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_RUST / "Cargo.toml.tera").read_text(encoding="utf-8")

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_actix_web_dependency(self) -> None:
        assert "actix-web" in self.content

    def test_async_graphql_dependency(self) -> None:
        assert "async-graphql" in self.content

    def test_serde_dependency(self) -> None:
        assert "serde" in self.content


class TestRustBffConfigContent:
    """Rust BFF config/config.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_RUST / "config" / "config.yaml.tera").read_text(
            encoding="utf-8"
        )

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_upstream_section(self) -> None:
        assert "upstream:" in self.content


class TestRustBffDockerfileContent:
    """Rust BFF Dockerfile.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_RUST / "Dockerfile.tera").read_text(encoding="utf-8")

    def test_multistage_build(self) -> None:
        assert "AS builder" in self.content

    def test_rust_base_image(self) -> None:
        assert "rust:" in self.content

    def test_distroless_runtime(self) -> None:
        assert "distroless" in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content


class TestRustBffReadmeContent:
    """Rust BFF README.md.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (BFF_RUST / "README.md.tera").read_text(encoding="utf-8")

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_bff_mention(self) -> None:
        assert "BFF" in self.content or "bff" in self.content


# ============================================================================
# Go BFF 新規テンプレートファイル存在確認
# ============================================================================


class TestGoBffUpstreamClientFile:
    """Go BFF upstream client テンプレートファイルの存在と内容検証。"""

    def test_upstream_client_exists(self) -> None:
        path = BFF_GO / "internal" / "client" / "upstream.go.tera"
        assert path.exists(), "bff/go/internal/client/upstream.go.tera が存在しません"

    def test_upstream_client_has_struct(self) -> None:
        content = (BFF_GO / "internal" / "client" / "upstream.go.tera").read_text(
            encoding="utf-8"
        )
        assert "UpstreamClient" in content

    def test_upstream_client_has_get(self) -> None:
        content = (BFF_GO / "internal" / "client" / "upstream.go.tera").read_text(
            encoding="utf-8"
        )
        assert "func (c *UpstreamClient) Get(" in content

    def test_upstream_client_has_post(self) -> None:
        content = (BFF_GO / "internal" / "client" / "upstream.go.tera").read_text(
            encoding="utf-8"
        )
        assert "func (c *UpstreamClient) Post(" in content


class TestGoBffResolverTestFile:
    """Go BFF resolver test テンプレートファイルの存在と内容検証。"""

    def test_resolver_test_exists(self) -> None:
        path = BFF_GO / "internal" / "handler" / "graphql_resolver_test.go.tera"
        assert path.exists(), "bff/go/internal/handler/graphql_resolver_test.go.tera が存在しません"

    def test_resolver_test_has_test_functions(self) -> None:
        content = (
            BFF_GO / "internal" / "handler" / "graphql_resolver_test.go.tera"
        ).read_text(encoding="utf-8")
        assert "TestNewResolver" in content
        assert "TestResolverQuery" in content
        assert "TestResolverMutation" in content


# ============================================================================
# Rust BFF 新規テンプレートファイル存在確認
# ============================================================================


class TestRustBffUpstreamClientFile:
    """Rust BFF upstream client テンプレートファイルの存在と内容検証。"""

    def test_upstream_client_exists(self) -> None:
        path = BFF_RUST / "src" / "client" / "upstream.rs.tera"
        assert path.exists(), "bff/rust/src/client/upstream.rs.tera が存在しません"

    def test_client_mod_exists(self) -> None:
        path = BFF_RUST / "src" / "client" / "mod.rs.tera"
        assert path.exists(), "bff/rust/src/client/mod.rs.tera が存在しません"

    def test_upstream_client_has_struct(self) -> None:
        content = (BFF_RUST / "src" / "client" / "upstream.rs.tera").read_text(
            encoding="utf-8"
        )
        assert "UpstreamClient" in content

    def test_upstream_client_has_get(self) -> None:
        content = (BFF_RUST / "src" / "client" / "upstream.rs.tera").read_text(
            encoding="utf-8"
        )
        assert "pub async fn get(" in content

    def test_upstream_client_has_post(self) -> None:
        content = (BFF_RUST / "src" / "client" / "upstream.rs.tera").read_text(
            encoding="utf-8"
        )
        assert "pub async fn post(" in content


class TestRustBffIntegrationTestFile:
    """Rust BFF integration test テンプレートファイルの存在と内容検証。"""

    def test_integration_test_exists(self) -> None:
        path = BFF_RUST / "tests" / "integration_test.rs.tera"
        assert path.exists(), "bff/rust/tests/integration_test.rs.tera が存在しません"

    def test_integration_test_has_schema_test(self) -> None:
        content = (BFF_RUST / "tests" / "integration_test.rs.tera").read_text(
            encoding="utf-8"
        )
        assert "test_schema_creation" in content
        assert "build_schema" in content


class TestRustBffHandlerModFile:
    """Rust BFF handler/mod.rs.tera テンプレートファイルの存在と内容検証。"""

    def test_handler_mod_exists(self) -> None:
        path = BFF_RUST / "src" / "handler" / "mod.rs.tera"
        assert path.exists(), "bff/rust/src/handler/mod.rs.tera が存在しません"

    def test_handler_mod_has_graphql(self) -> None:
        content = (BFF_RUST / "src" / "handler" / "mod.rs.tera").read_text(
            encoding="utf-8"
        )
        assert "pub mod graphql;" in content


# ============================================================================
# BFF config upstream.grpc_address 検証
# ============================================================================


class TestBffConfigUpstreamGrpc:
    """BFF config に upstream.grpc_address が含まれることの検証。"""

    def test_go_config_has_grpc_address(self) -> None:
        content = (BFF_GO / "config" / "config.yaml.tera").read_text(encoding="utf-8")
        assert "grpc_address:" in content, "Go BFF config should have grpc_address"

    def test_rust_config_has_grpc_address(self) -> None:
        content = (BFF_RUST / "config" / "config.yaml.tera").read_text(
            encoding="utf-8"
        )
        assert "grpc_address:" in content, "Rust BFF config should have grpc_address"

    def test_go_config_has_http_url(self) -> None:
        content = (BFF_GO / "config" / "config.yaml.tera").read_text(encoding="utf-8")
        assert "http_url:" in content, "Go BFF config should have http_url"

    def test_rust_config_has_http_url(self) -> None:
        content = (BFF_RUST / "config" / "config.yaml.tera").read_text(
            encoding="utf-8"
        )
        assert "http_url:" in content, "Rust BFF config should have http_url"


# ============================================================================
# Go BFF GraphQL resolver Query/Mutation 内容検証
# ============================================================================


class TestGoBffResolverQueryMutation:
    """Go BFF resolver に Query/Mutation 実装が含まれることの検証。"""

    def setup_method(self) -> None:
        self.content = (
            BFF_GO / "internal" / "handler" / "graphql_resolver.go.tera"
        ).read_text(encoding="utf-8")

    def test_has_query_method(self) -> None:
        assert "func (r *Resolver) Query()" in self.content

    def test_has_mutation_method(self) -> None:
        assert "func (r *Resolver) Mutation()" in self.content

    def test_has_upstream_import(self) -> None:
        assert "client" in self.content

    def test_has_upstream_field(self) -> None:
        assert "upstream" in self.content


# ============================================================================
# Rust BFF GraphQL handler async-graphql Schema 内容検証
# ============================================================================


class TestRustBffGraphqlSchemaContent:
    """Rust BFF graphql.rs.tera に async-graphql Schema 構築コードが含まれることの検証。"""

    def setup_method(self) -> None:
        self.content = (
            BFF_RUST / "src" / "handler" / "graphql.rs.tera"
        ).read_text(encoding="utf-8")

    def test_has_query_root(self) -> None:
        assert "pub struct QueryRoot" in self.content

    def test_has_mutation_root(self) -> None:
        assert "pub struct MutationRoot" in self.content

    def test_has_build_schema(self) -> None:
        assert "pub fn build_schema(" in self.content

    def test_has_schema_type(self) -> None:
        assert "BffSchema" in self.content

    def test_has_upstream_client(self) -> None:
        assert "UpstreamClient" in self.content


# ============================================================================
# Go BFF schema.graphql Query/Mutation 検証
# ============================================================================


class TestGoBffSchemaQueryMutation:
    """Go BFF schema.graphql.tera に Query/Mutation が含まれることの検証。"""

    def setup_method(self) -> None:
        self.content = (
            BFF_GO / "api" / "graphql" / "schema.graphql.tera"
        ).read_text(encoding="utf-8")

    def test_has_query_type(self) -> None:
        assert "type Query" in self.content

    def test_has_mutation_type(self) -> None:
        assert "type Mutation" in self.content

    def test_has_service_name_pascal(self) -> None:
        assert "{{ service_name_pascal }}" in self.content

    def test_has_create_input(self) -> None:
        assert "Create{{ service_name_pascal }}Input" in self.content
