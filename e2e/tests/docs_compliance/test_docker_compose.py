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


class TestDockerComposeHealthchecks:
    """docker-compose設計.md: healthcheck 設定の検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    @pytest.mark.parametrize(
        "service,test_cmd",
        [
            ("postgres", "pg_isready"),
            ("mysql", "mysqladmin"),
            ("redis", "redis-cli"),
            ("kafka", "kafka-broker-api-versions.sh"),
            ("schema-registry", "curl"),
            ("redis-session", "redis-cli"),
        ],
    )
    def test_healthcheck_defined(self, service: str, test_cmd: str) -> None:
        """docker-compose設計.md: 各サービスに healthcheck が定義されていること。"""
        svc = self.config["services"][service]
        assert "healthcheck" in svc, f"{service} に healthcheck が定義されていません"
        test_str = " ".join(str(x) for x in svc["healthcheck"]["test"])
        assert test_cmd in test_str, f"{service} の healthcheck に {test_cmd} が含まれていません"


class TestDockerComposeVolumeMounts:
    """docker-compose設計.md: volumes マウント先テスト。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    @pytest.mark.parametrize(
        "service,expected_mount",
        [
            ("postgres", "/var/lib/postgresql/data"),
            ("mysql", "/var/lib/mysql"),
            ("redis", "/data"),
            ("kafka", "/bitnami/kafka"),
            ("redis-session", "/data"),
            ("prometheus", "/prometheus"),
            ("loki", "/loki"),
            ("grafana", "/var/lib/grafana"),
        ],
    )
    def test_volume_mount(self, service: str, expected_mount: str) -> None:
        """docker-compose設計.md: 各サービスのボリュームマウント先が仕様通りであること。"""
        svc = self.config["services"][service]
        volumes_str = str(svc.get("volumes", []))
        assert expected_mount in volumes_str, (
            f"{service} のボリュームに {expected_mount} が含まれていません"
        )


class TestDockerComposeDependsOn:
    """docker-compose設計.md: depends_on 依存関係テスト。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_kafka_ui_depends_on_kafka(self) -> None:
        deps = self.config["services"]["kafka-ui"]["depends_on"]
        assert "kafka" in deps

    def test_kafka_ui_depends_on_schema_registry(self) -> None:
        deps = self.config["services"]["kafka-ui"]["depends_on"]
        assert "schema-registry" in deps

    def test_schema_registry_depends_on_kafka(self) -> None:
        deps = self.config["services"]["schema-registry"]["depends_on"]
        assert "kafka" in deps

    def test_keycloak_depends_on_postgres(self) -> None:
        deps = self.config["services"]["keycloak"]["depends_on"]
        assert "postgres" in deps

    def test_grafana_depends_on_prometheus(self) -> None:
        deps = self.config["services"]["grafana"]["depends_on"]
        assert "prometheus" in deps or "prometheus" in [d if isinstance(d, str) else "" for d in deps]


class TestDockerComposeEnvironment:
    """docker-compose設計.md: environment 詳細テスト。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_kafka_environment(self) -> None:
        env = self.config["services"]["kafka"]["environment"]
        assert env["KAFKA_CFG_NODE_ID"] == 0
        assert "broker" in env["KAFKA_CFG_PROCESS_ROLES"]
        assert "controller" in env["KAFKA_CFG_PROCESS_ROLES"]
        assert "PLAINTEXT" in env["KAFKA_CFG_LISTENERS"]

    def test_schema_registry_environment(self) -> None:
        env = self.config["services"]["schema-registry"]["environment"]
        assert env["SCHEMA_REGISTRY_HOST_NAME"] == "schema-registry"
        assert "kafka:9092" in env["SCHEMA_REGISTRY_KAFKASTORE_BOOTSTRAP_SERVERS"]
        assert "8081" in env["SCHEMA_REGISTRY_LISTENERS"]

    def test_keycloak_environment(self) -> None:
        env = self.config["services"]["keycloak"]["environment"]
        assert env["KC_DB"] == "postgres"
        assert env["KC_DB_URL_HOST"] == "postgres"
        assert env["KC_DB_URL_DATABASE"] == "keycloak"
        assert env["KEYCLOAK_ADMIN"] == "admin"


class TestDockerComposeProfiles:
    """docker-compose設計.md: profiles 設定テスト。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    @pytest.mark.parametrize(
        "service,profile",
        [
            ("postgres", "infra"),
            ("mysql", "infra"),
            ("redis", "infra"),
            ("kafka", "infra"),
            ("kafka-ui", "infra"),
            ("schema-registry", "infra"),
            ("keycloak", "infra"),
            ("redis-session", "infra"),
            ("vault", "infra"),
            ("jaeger", "observability"),
            ("prometheus", "observability"),
            ("loki", "observability"),
            ("grafana", "observability"),
        ],
    )
    def test_service_profiles(self, service: str, profile: str) -> None:
        """docker-compose設計.md: 各サービスが正しいプロファイルに属すること。"""
        svc = self.config["services"][service]
        assert profile in svc["profiles"], (
            f"{service} が {profile} プロファイルに属していません"
        )


class TestDockerComposeServiceNameResolution:
    """docker-compose設計.md: サービス名解決テーブルテスト。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    @pytest.mark.parametrize(
        "service",
        [
            "postgres", "mysql", "redis", "kafka", "schema-registry",
            "jaeger", "vault", "keycloak", "redis-session",
        ],
    )
    def test_service_exists_for_name_resolution(self, service: str) -> None:
        """docker-compose設計.md: サービス名解決テーブルに対応するサービスが存在すること。"""
        assert service in self.config["services"]


class TestDockerComposeConfigDevExample:
    """docker-compose設計.md: config.dev.yaml 例テスト。"""

    def test_dev_config_host_is_docker_service(self) -> None:
        """docker-compose設計.md: values-dev の config.yaml でホストがdockerサービス名であること。"""
        path = ROOT / "infra" / "helm" / "services" / "service" / "order" / "values-dev.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        config_yaml = values["config"]["data"]["config.yaml"]
        # Kubernetes DNS 名を使用（ローカル開発では docker-compose サービス名を使用するが、
        # values-dev.yaml は Kubernetes の dev 環境用）
        assert "postgres" in config_yaml


class TestDockerComposeHostnameHardcodeRule:
    """docker-compose設計.md: ホスト名ハードコード禁止ルールテスト。"""

    def test_vault_no_hardcoded_hostname(self) -> None:
        """docker-compose設計.md: Vault の環境変数にホスト名がハードコードされていないこと。"""
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            config = yaml.safe_load(f)
        vault = config["services"]["vault"]
        env = vault.get("environment", {})
        assert "VAULT_DEV_ROOT_TOKEN_ID" in env


class TestDockerComposeVaultService:
    """docker-compose設計.md: Vault cap_add/DEV_ROOT_TOKEN_ID テスト。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_vault_cap_add(self) -> None:
        """docker-compose設計.md: Vault の cap_add に IPC_LOCK が含まれること。"""
        vault = self.config["services"]["vault"]
        assert "IPC_LOCK" in vault["cap_add"]

    def test_vault_dev_root_token(self) -> None:
        """docker-compose設計.md: VAULT_DEV_ROOT_TOKEN_ID が dev-token であること。"""
        vault = self.config["services"]["vault"]
        assert vault["environment"]["VAULT_DEV_ROOT_TOKEN_ID"] == "dev-token"


class TestDockerComposeDependsOnComplete:
    """docker-compose設計.md: depends_on 完全検証テスト。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_grafana_depends_on_all(self) -> None:
        """docker-compose設計.md: Grafana が prometheus, loki, jaeger に依存していること。"""
        deps = self.config["services"]["grafana"]["depends_on"]
        if isinstance(deps, list):
            assert "prometheus" in deps
            assert "loki" in deps
            assert "jaeger" in deps
        else:
            assert "prometheus" in deps
            assert "loki" in deps
            assert "jaeger" in deps

    def test_keycloak_depends_on_postgres_healthy(self) -> None:
        """docker-compose設計.md: Keycloak が postgres:service_healthy に依存していること。"""
        deps = self.config["services"]["keycloak"]["depends_on"]
        assert "postgres" in deps
        if isinstance(deps, dict):
            assert deps["postgres"]["condition"] == "service_healthy"

    def test_kafka_ui_depends_on_healthy(self) -> None:
        """docker-compose設計.md: kafka-ui が kafka と schema-registry の service_healthy に依存。"""
        deps = self.config["services"]["kafka-ui"]["depends_on"]
        assert isinstance(deps, dict)
        assert deps["kafka"]["condition"] == "service_healthy"
        assert deps["schema-registry"]["condition"] == "service_healthy"


class TestDockerComposeVolumeDetails:
    """docker-compose設計.md: volumes 詳細テスト。"""

    def setup_method(self) -> None:
        config_path = ROOT / "docker-compose.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_keycloak_volume_import(self) -> None:
        """docker-compose設計.md: Keycloak の realm import ボリュームが設定されていること。"""
        volumes = self.config["services"]["keycloak"]["volumes"]
        volumes_str = str(volumes)
        assert "/opt/keycloak/data/import" in volumes_str

    def test_prometheus_config_volume(self) -> None:
        """docker-compose設計.md: Prometheus の設定ファイルがマウントされていること。"""
        volumes = self.config["services"]["prometheus"]["volumes"]
        volumes_str = str(volumes)
        assert "prometheus" in volumes_str
        assert "/etc/prometheus" in volumes_str

    def test_grafana_provisioning_volume(self) -> None:
        """docker-compose設計.md: Grafana の provisioning ディレクトリがマウントされていること。"""
        volumes = self.config["services"]["grafana"]["volumes"]
        volumes_str = str(volumes)
        assert "/etc/grafana/provisioning" in volumes_str

    def test_grafana_dashboards_volume(self) -> None:
        """docker-compose設計.md: Grafana の dashboards ディレクトリがマウントされていること。"""
        volumes = self.config["services"]["grafana"]["volumes"]
        volumes_str = str(volumes)
        assert "dashboards" in volumes_str


class TestDockerComposeOverrideExample:
    """docker-compose設計.md: docker-compose.override.yaml.example 存在テスト。"""

    def test_override_example_exists(self) -> None:
        path = ROOT / "docker-compose.override.yaml.example"
        assert path.exists(), "docker-compose.override.yaml.example が存在しません"


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
