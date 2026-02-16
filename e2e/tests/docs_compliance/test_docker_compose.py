"""docker-compose設計.md の仕様準拠テスト。

docker-compose.yaml の各サービス定義がドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]


class TestDockerComposeServices:
    """docker-compose設計.md: サービス定義の検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        assert config_path.exists(), "docker-compose.yaml が存在しません"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_postgres_service(self) -> None:
        svc = self.config["services"]["postgres"]
        assert svc["image"] == "postgres:17"
        assert svc["profiles"] == ["infra"]
        assert svc["environment"]["POSTGRES_USER"] == "dev"
        assert "5432:5432" in svc["ports"]

    def test_mysql_service(self) -> None:
        svc = self.config["services"]["mysql"]
        assert svc["image"] == "mysql:8.4"
        assert svc["profiles"] == ["infra"]
        assert "3306:3306" in svc["ports"]

    def test_redis_service(self) -> None:
        svc = self.config["services"]["redis"]
        assert svc["image"] == "redis:7"
        assert svc["profiles"] == ["infra"]
        assert "6379:6379" in svc["ports"]

    def test_kafka_service(self) -> None:
        svc = self.config["services"]["kafka"]
        assert svc["image"] == "bitnami/kafka:3.8"
        assert svc["profiles"] == ["infra"]
        assert "9092:9092" in svc["ports"]

    def test_kafka_ui_service(self) -> None:
        svc = self.config["services"]["kafka-ui"]
        assert svc["image"] == "provectuslabs/kafka-ui:latest"
        assert svc["profiles"] == ["infra"]
        assert "8090:8080" in svc["ports"]

    def test_schema_registry_service(self) -> None:
        svc = self.config["services"]["schema-registry"]
        assert svc["image"] == "confluentinc/cp-schema-registry:7.7"
        assert svc["profiles"] == ["infra"]
        assert "8081:8081" in svc["ports"]

    def test_keycloak_service(self) -> None:
        svc = self.config["services"]["keycloak"]
        assert svc["image"] == "quay.io/keycloak/keycloak:26.0"
        assert svc["profiles"] == ["infra"]
        assert "8180:8080" in svc["ports"]
        assert svc["command"] == "start-dev --import-realm"

    def test_redis_session_service(self) -> None:
        svc = self.config["services"]["redis-session"]
        assert svc["image"] == "redis:7"
        assert svc["profiles"] == ["infra"]
        assert "6380:6379" in svc["ports"]

    def test_vault_service(self) -> None:
        svc = self.config["services"]["vault"]
        assert svc["image"] == "hashicorp/vault:1.17"
        assert svc["profiles"] == ["infra"]
        assert "8200:8200" in svc["ports"]

    def test_jaeger_service(self) -> None:
        svc = self.config["services"]["jaeger"]
        assert svc["image"] == "jaegertracing/all-in-one:1.62"
        assert svc["profiles"] == ["observability"]
        assert "16686:16686" in svc["ports"]
        assert "4317:4317" in svc["ports"]
        assert "4318:4318" in svc["ports"]

    def test_prometheus_service(self) -> None:
        svc = self.config["services"]["prometheus"]
        assert svc["image"] == "prom/prometheus:v2.55"
        assert svc["profiles"] == ["observability"]
        assert "9090:9090" in svc["ports"]

    def test_loki_service(self) -> None:
        svc = self.config["services"]["loki"]
        assert svc["image"] == "grafana/loki:3.3"
        assert svc["profiles"] == ["observability"]
        assert "3100:3100" in svc["ports"]

    def test_grafana_service(self) -> None:
        svc = self.config["services"]["grafana"]
        assert svc["image"] == "grafana/grafana:11.3"
        assert svc["profiles"] == ["observability"]
        assert "3200:3000" in svc["ports"]

    def test_all_services_count(self) -> None:
        expected_services = [
            "postgres", "mysql", "redis", "kafka", "kafka-ui",
            "schema-registry", "keycloak", "redis-session", "vault",
            "jaeger", "prometheus", "loki", "grafana",
        ]
        for svc in expected_services:
            assert svc in self.config["services"], f"サービス {svc} が存在しません"

    def test_volumes(self) -> None:
        expected_volumes = [
            "postgres-data", "mysql-data", "redis-data",
            "redis-session-data", "kafka-data",
            "prometheus-data", "loki-data", "grafana-data",
        ]
        for vol in expected_volumes:
            assert vol in self.config["volumes"], f"ボリューム {vol} が存在しません"

    def test_network(self) -> None:
        assert self.config["networks"]["default"]["name"] == "k1s0-network"


class TestInitDBScript:
    """docker-compose設計.md: DB初期化スクリプトの検証。"""

    def test_init_db_script_exists(self) -> None:
        script = ROOT / "infra" / "docker" / "init-db" / "01-create-databases.sql"
        assert script.exists(), "01-create-databases.sql が存在しません"

    def test_init_db_creates_databases(self) -> None:
        script = ROOT / "infra" / "docker" / "init-db" / "01-create-databases.sql"
        content = script.read_text(encoding="utf-8")
        assert "CREATE DATABASE keycloak" in content
        assert "CREATE DATABASE kong" in content
        assert "CREATE DATABASE k1s0_system" in content
        assert "CREATE DATABASE k1s0_business" in content
        assert "CREATE DATABASE k1s0_service" in content
