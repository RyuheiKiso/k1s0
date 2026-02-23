"""コンセプト.md の仕様準拠テスト。

docs/コンセプト.md で定義された技術スタック・設計思想が
プロジェクト内のファイル（テンプレート、設定ファイル等）に正しく反映されているかを検証する。
"""

from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"
CLI_SRC = ROOT / "CLI" / "crates" / "k1s0-cli" / "src"
DOCS = ROOT / "docs"


# ============================================================================
# 1. 設計思想の検証
# ============================================================================


class TestDesignPhilosophy:
    """コンセプト.md: 設計思想（クリーンアーキテクチャ・DDD・TDD・マイクロサービス・イベント駆動）の検証。"""

    def test_concept_mentions_clean_architecture(self) -> None:
        """コンセプト.md: クリーンアーキテクチャが記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert "クリーンアーキテクチャ" in content

    def test_concept_mentions_ddd(self) -> None:
        """コンセプト.md: DDD が記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert "ドメイン駆動設計" in content or "DDD" in content

    def test_concept_mentions_tdd(self) -> None:
        """コンセプト.md: TDD が記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert "テスト駆動開発" in content or "TDD" in content

    def test_concept_mentions_microservices(self) -> None:
        """コンセプト.md: マイクロサービスアーキテクチャが記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert "マイクロサービス" in content

    def test_concept_mentions_event_driven(self) -> None:
        """コンセプト.md: イベント駆動アーキテクチャが記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert "イベント駆動" in content

    def test_clean_architecture_reflected_in_go_server(self) -> None:
        """コンセプト.md: Go サーバーテンプレートがクリーンアーキテクチャ構成（domain/usecase/adapter/infra）を持つ。"""
        for layer in ["domain", "usecase", "adapter", "infra"]:
            assert (TEMPLATES / "server" / "go" / "internal" / layer).exists(), (
                f"Go サーバーテンプレートに {layer} 層がありません"
            )

    def test_clean_architecture_reflected_in_rust_server(self) -> None:
        """コンセプト.md: Rust サーバーテンプレートがクリーンアーキテクチャ構成（domain/usecase/adapter/infra）を持つ。"""
        for layer in ["domain", "usecase", "adapter", "infra"]:
            assert (TEMPLATES / "server" / "rust" / "src" / layer).exists(), (
                f"Rust サーバーテンプレートに {layer} 層がありません"
            )


# ============================================================================
# 2. 設計パターン（CQRS、イベントソーシング、Saga）の検証
# ============================================================================


class TestDesignPatterns:
    """コンセプト.md: 設計パターンの記載検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_cqrs_pattern_documented(self) -> None:
        """コンセプト.md: CQRS パターンが記載されている。"""
        assert "CQRS" in self.content

    def test_event_sourcing_pattern_documented(self) -> None:
        """コンセプト.md: イベントソーシングパターンが記載されている。"""
        assert "イベントソーシング" in self.content

    def test_saga_pattern_documented(self) -> None:
        """コンセプト.md: Saga パターンが記載されている。"""
        assert "Saga" in self.content

    def test_pattern_application_criteria(self) -> None:
        """コンセプト.md: パターン適用基準が記載されている。"""
        assert "適用基準" in self.content
        assert "適用条件" in self.content


# ============================================================================
# 3. API/通信方式の技術スタック検証
# ============================================================================


class TestApiCommunicationStack:
    """コンセプト.md: API/通信方式の技術スタック検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_gqlgen_documented(self) -> None:
        """コンセプト.md: gqlgen（Go向けGraphQL）が記載されている。"""
        assert "gqlgen" in self.content

    def test_async_graphql_documented(self) -> None:
        """コンセプト.md: async-graphql（Rust向けGraphQL）が記載されている。"""
        assert "async-graphql" in self.content

    def test_openapi_documented(self) -> None:
        """コンセプト.md: OpenAPI が記載されている。"""
        assert "OpenAPI" in self.content

    def test_protobuf_documented(self) -> None:
        """コンセプト.md: protobuf が記載されている。"""
        assert "protobuf" in self.content

    def test_buf_documented(self) -> None:
        """コンセプト.md: buf が記載されている。"""
        assert "buf" in self.content

    def test_rest_api_template_exists(self) -> None:
        """コンセプト.md: REST API テンプレート（OpenAPI定義）が存在する。"""
        assert (TEMPLATES / "server" / "go" / "api" / "openapi" / "openapi.yaml.tera").exists()

    def test_grpc_template_exists(self) -> None:
        """コンセプト.md: gRPC テンプレート（proto定義）が存在する。"""
        assert (TEMPLATES / "server" / "go" / "api" / "proto" / "service.proto.tera").exists()

    def test_graphql_template_exists(self) -> None:
        """コンセプト.md: GraphQL テンプレート（schema定義）が存在する。"""
        assert (TEMPLATES / "server" / "go" / "api" / "graphql" / "schema.graphql.tera").exists()


# ============================================================================
# 4. セキュリティ技術スタック検証
# ============================================================================


class TestSecurityStack:
    """コンセプト.md: セキュリティ技術スタック検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_oauth_oidc_documented(self) -> None:
        """コンセプト.md: OAuth 2.0 / OIDC が記載されている。"""
        assert "OAuth 2.0" in self.content
        assert "OIDC" in self.content

    def test_jwt_documented(self) -> None:
        """コンセプト.md: JWT が記載されている。"""
        assert "JWT" in self.content

    def test_keycloak_version_documented(self) -> None:
        """コンセプト.md: Keycloak 26.0 LTS が記載されている。"""
        assert "Keycloak 26.0 LTS" in self.content

    def test_vault_version_documented(self) -> None:
        """コンセプト.md: HashiCorp Vault 1.17 が記載されている。"""
        assert "Vault 1.17" in self.content


# ============================================================================
# 5. インフラ技術スタック検証
# ============================================================================


class TestInfraStack:
    """コンセプト.md: インフラ技術スタック検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "tech",
        [
            "Docker",
            "Kubernetes",
            "Helm",
            "Terraform",
            "Ansible",
            "GitHub Actions",
            "MetalLB",
            "Calico",
            "cert-manager",
            "Ceph",
            "Consul",
            "Harbor",
            "Cosign",
            "Flagger",
        ],
    )
    def test_infra_tech_documented(self, tech: str) -> None:
        """コンセプト.md: インフラ技術が記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert tech in content, f"{tech} がコンセプト.md に記載されていません"

    def test_terraform_dir_exists(self) -> None:
        """コンセプト.md: Terraform ディレクトリが存在する。"""
        assert (ROOT / "infra" / "terraform").exists()

    def test_helm_dir_exists(self) -> None:
        """コンセプト.md: Helm ディレクトリが存在する。"""
        assert (ROOT / "infra" / "helm").exists()

    def test_docker_compose_exists(self) -> None:
        """コンセプト.md: docker-compose.yaml が存在する。"""
        assert (ROOT / "docker-compose.yaml").exists()


# ============================================================================
# 6. サービスメッシュ検証
# ============================================================================


class TestServiceMeshStack:
    """コンセプト.md: サービスメッシュ（Istio 1.24/Envoy、Kiali）検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_istio_version_documented(self) -> None:
        """コンセプト.md: Istio 1.24 が記載されている。"""
        assert "Istio 1.24" in self.content

    def test_envoy_documented(self) -> None:
        """コンセプト.md: Envoy が記載されている。"""
        assert "Envoy" in self.content

    def test_kiali_documented(self) -> None:
        """コンセプト.md: Kiali が記載されている。"""
        assert "Kiali" in self.content

    def test_istio_config_exists(self) -> None:
        """コンセプト.md: Istio 設定ファイルが存在する。"""
        assert (ROOT / "infra" / "istio").exists()


# ============================================================================
# 7. 可観測性スタック検証
# ============================================================================


class TestObservabilityStack:
    """コンセプト.md: 可観測性スタック検証。"""

    @pytest.mark.parametrize(
        "tech",
        [
            "OpenTelemetry",
            "Jaeger",
            "Prometheus",
            "Alertmanager",
            "Grafana",
            "Loki",
            "Promtail",
        ],
    )
    def test_observability_tech_documented(self, tech: str) -> None:
        """コンセプト.md: 可観測性技術が記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert tech in content, f"{tech} がコンセプト.md に記載されていません"

    def test_prometheus_config_exists(self) -> None:
        """コンセプト.md: Prometheus 設定ディレクトリが存在する。"""
        assert (ROOT / "infra" / "docker" / "prometheus").exists()

    def test_grafana_config_exists(self) -> None:
        """コンセプト.md: Grafana 設定ディレクトリが存在する。"""
        assert (ROOT / "infra" / "docker" / "grafana").exists()


# ============================================================================
# 8. メッセージング検証
# ============================================================================


class TestMessagingStack:
    """コンセプト.md: メッセージング（Kafka 3.8、Strimzi）検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_kafka_version_documented(self) -> None:
        """コンセプト.md: Kafka 3.8 が記載されている。"""
        assert "Kafka 3.8" in self.content

    def test_strimzi_documented(self) -> None:
        """コンセプト.md: Strimzi が記載されている。"""
        assert "Strimzi" in self.content

    def test_schema_registry_documented(self) -> None:
        """コンセプト.md: Confluent Schema Registry が記載されている。"""
        assert "Schema Registry" in self.content

    def test_kafka_template_exists(self) -> None:
        """コンセプト.md: Kafka テンプレート（messaging）が Go サーバーに存在する。"""
        assert (
            TEMPLATES / "server" / "go" / "internal" / "infra" / "messaging" / "kafka.go.tera"
        ).exists()

    def test_kafka_template_exists_rust(self) -> None:
        """コンセプト.md: Kafka テンプレート（messaging）が Rust サーバーに存在する。"""
        assert (TEMPLATES / "server" / "rust" / "src" / "infra" / "messaging.rs.tera").exists()


# ============================================================================
# 9. ORM/クエリビルダー（sqlx）検証
# ============================================================================


class TestOrmQueryBuilder:
    """コンセプト.md: ORM/クエリビルダー（sqlx）検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_sqlx_documented(self) -> None:
        """コンセプト.md: sqlx が記載されている。"""
        assert "sqlx" in self.content

    def test_go_sqlx_documented(self) -> None:
        """コンセプト.md: Go 1.23 + sqlx が記載されている。"""
        assert "Go 1.23" in self.content

    def test_rust_sqlx_documented(self) -> None:
        """コンセプト.md: Rust 1.82 + sqlx が記載されている。"""
        assert "Rust 1.82" in self.content

    def test_go_persistence_template_exists(self) -> None:
        """コンセプト.md: Go サーバーに persistence テンプレートが存在する。"""
        assert (
            TEMPLATES / "server" / "go" / "internal" / "infra" / "persistence" / "db.go.tera"
        ).exists()

    def test_rust_persistence_template_exists(self) -> None:
        """コンセプト.md: Rust サーバーに persistence テンプレートが存在する。"""
        assert (TEMPLATES / "server" / "rust" / "src" / "infra" / "persistence.rs.tera").exists()


# ============================================================================
# 10. データベースマイグレーション検証
# ============================================================================


class TestDatabaseMigration:
    """コンセプト.md: データベースマイグレーション（golang-migrate / sqlx-cli）検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_golang_migrate_documented(self) -> None:
        """コンセプト.md: golang-migrate が記載されている。"""
        assert "golang-migrate" in self.content

    def test_sqlx_cli_documented(self) -> None:
        """コンセプト.md: sqlx-cli が記載されている。"""
        assert "sqlx-cli" in self.content

    def test_migration_template_format(self) -> None:
        """コンセプト.md: マイグレーションテンプレートが up/down 形式で存在する。"""
        assert (TEMPLATES / "database" / "postgresql" / "001_init.up.sql.tera").exists()
        assert (TEMPLATES / "database" / "postgresql" / "001_init.down.sql.tera").exists()


# ============================================================================
# 11. API ゲートウェイ（Kong 3.7）検証
# ============================================================================


class TestApiGateway:
    """コンセプト.md: APIゲートウェイ（Kong 3.7）検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_kong_version_documented(self) -> None:
        """コンセプト.md: Kong 3.7 が記載されている。"""
        assert "Kong 3.7" in self.content

    def test_kong_config_exists(self) -> None:
        """コンセプト.md: Kong 設定ファイルが存在する。"""
        assert (ROOT / "infra" / "kong" / "kong.yaml").exists()


# ============================================================================
# 12. 耐障害性パターン検証
# ============================================================================


class TestResiliencePatterns:
    """コンセプト.md: 耐障害性パターン（Circuit Breaker、Retry、Rate Limiting）検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_circuit_breaker_documented(self) -> None:
        """コンセプト.md: Circuit Breaker が記載されている。"""
        assert "Circuit Breaker" in self.content

    def test_retry_documented(self) -> None:
        """コンセプト.md: Retry が記載されている。"""
        assert "Retry" in self.content

    def test_rate_limiting_documented(self) -> None:
        """コンセプト.md: Rate Limiting が記載されている。"""
        assert "Rate Limiting" in self.content

    def test_circuit_breaker_impl_istio(self) -> None:
        """コンセプト.md: Circuit Breaker の実装主体が Istio であることが記載されている。"""
        assert "Circuit Breaker" in self.content
        assert "Istio" in self.content

    def test_rate_limiting_impl_kong(self) -> None:
        """コンセプト.md: Rate Limiting の実装主体が Kong であることが記載されている。"""
        assert "Rate Limiting" in self.content
        assert "Kong" in self.content


# ============================================================================
# 13. 開発者体験（Dev Container、docker-compose）検証
# ============================================================================


class TestDeveloperExperience:
    """コンセプト.md: 開発者体験（Dev Container、docker-compose）検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_devcontainer_documented(self) -> None:
        """コンセプト.md: Dev Container が記載されている。"""
        assert "Dev Container" in self.content

    def test_docker_compose_documented(self) -> None:
        """コンセプト.md: docker-compose が記載されている。"""
        assert "docker-compose" in self.content

    def test_devcontainer_json_exists(self) -> None:
        """コンセプト.md: devcontainer.json が存在する。"""
        assert (ROOT / ".devcontainer" / "devcontainer.json").exists()

    def test_docker_compose_yaml_exists(self) -> None:
        """コンセプト.md: docker-compose.yaml が存在する。"""
        assert (ROOT / "docker-compose.yaml").exists()


# ============================================================================
# 14. React クライアント技術スタック検証
# ============================================================================


class TestReactClientStack:
    """コンセプト.md: React クライアント技術スタック検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "tech",
        [
            "TanStack Query",
            "Zustand",
            "TanStack Router",
            "React Hook Form",
            "Zod",
            "Tailwind CSS",
            "Radix UI",
        ],
    )
    def test_react_tech_documented(self, tech: str) -> None:
        """コンセプト.md: React クライアント技術が記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert tech in content, f"{tech} がコンセプト.md に記載されていません"

    def test_react_template_package_json_exists(self) -> None:
        """コンセプト.md: React テンプレートの package.json が存在する。"""
        assert (TEMPLATES / "client" / "react" / "package.json.tera").exists()


# ============================================================================
# 15. Flutter クライアント技術スタック検証
# ============================================================================


class TestFlutterClientStack:
    """コンセプト.md: Flutter クライアント技術スタック検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "tech",
        ["Riverpod", "go_router", "freezed", "dio"],
    )
    def test_flutter_tech_documented(self, tech: str) -> None:
        """コンセプト.md: Flutter クライアント技術が記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert tech in content, f"{tech} がコンセプト.md に記載されていません"

    def test_flutter_template_pubspec_exists(self) -> None:
        """コンセプト.md: Flutter テンプレートの pubspec.yaml が存在する。"""
        assert (TEMPLATES / "client" / "flutter" / "pubspec.yaml.tera").exists()


# ============================================================================
# 16. テスト技術スタック検証
# ============================================================================


class TestTestingStack:
    """コンセプト.md: テスト技術スタック検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "tech",
        [
            "testify",
            "gomock",
            "mockall",
            "Vitest",
            "Testing Library",
            "MSW",
            "mocktail",
            "pytest",
        ],
    )
    def test_testing_tech_documented(self, tech: str) -> None:
        """コンセプト.md: テスト技術が記載されている。"""
        content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")
        assert tech in content, f"{tech} がコンセプト.md に記載されていません"

    def test_go_test_template_exists(self) -> None:
        """コンセプト.md: Go テストテンプレートが存在する。"""
        assert (
            TEMPLATES / "server" / "go" / "internal" / "usecase" / "usecase_test.go.tera"
        ).exists()

    def test_rust_integration_test_template_exists(self) -> None:
        """コンセプト.md: Rust 統合テストテンプレートが存在する。"""
        assert (TEMPLATES / "server" / "rust" / "tests" / "integration_test.rs.tera").exists()

    def test_react_test_template_exists(self) -> None:
        """コンセプト.md: React テストテンプレートが存在する。"""
        assert (TEMPLATES / "client" / "react" / "tests" / "App.test.tsx.tera").exists()

    def test_flutter_test_template_exists(self) -> None:
        """コンセプト.md: Flutter テストテンプレートが存在する。"""
        assert (TEMPLATES / "client" / "flutter" / "test" / "widget_test.dart.tera").exists()


# ============================================================================
# 17. コード自動生成パイプライン検証
# ============================================================================


class TestCodeGenerationPipeline:
    """コンセプト.md: コード・ドキュメント自動生成パイプライン検証。"""

    def setup_method(self) -> None:
        self.content = (DOCS / "コンセプト.md").read_text(encoding="utf-8")

    def test_openapi_sdk_generation_documented(self) -> None:
        """コンセプト.md: OpenAPI -> SDK生成が記載されている。"""
        assert "OpenAPI" in self.content
        assert "SDK生成" in self.content or "自動生成" in self.content

    def test_protobuf_doc_generation_documented(self) -> None:
        """コンセプト.md: protobuf -> ドキュメント生成が記載されている。"""
        assert "protobuf" in self.content
        assert "ドキュメント生成" in self.content or "ドキュメント自動生成" in self.content

    def test_oapi_codegen_template_exists(self) -> None:
        """コンセプト.md: oapi-codegen 設定テンプレートが存在する（REST API コード生成）。"""
        assert (TEMPLATES / "server" / "go" / "oapi-codegen.yaml.tera").exists()

    def test_buf_gen_template_exists(self) -> None:
        """コンセプト.md: buf.gen.yaml テンプレートが存在する（gRPC コード生成）。"""
        assert (TEMPLATES / "server" / "go" / "buf.gen.yaml.tera").exists()

    def test_gqlgen_template_exists(self) -> None:
        """コンセプト.md: gqlgen.yml テンプレートが存在する（GraphQL コード生成）。"""
        assert (TEMPLATES / "server" / "go" / "gqlgen.yml.tera").exists()
