"""helm設計.md の仕様準拠テスト。

infra/helm/ の構成がドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]
HELM = ROOT / "infra" / "helm"


class TestHelmLibraryChart:
    """helm設計.md: k1s0-common Library Chart の検証。"""

    def test_chart_yaml_exists(self) -> None:
        assert (HELM / "charts" / "k1s0-common" / "Chart.yaml").exists()

    def test_chart_yaml_content(self) -> None:
        path = HELM / "charts" / "k1s0-common" / "Chart.yaml"
        with open(path, encoding="utf-8") as f:
            chart = yaml.safe_load(f)
        assert chart["apiVersion"] == "v2"
        assert chart["name"] == "k1s0-common"
        assert chart["type"] == "library"

    @pytest.mark.parametrize(
        "template",
        [
            "_deployment.tpl",
            "_service.tpl",
            "_hpa.tpl",
            "_pdb.tpl",
            "_configmap.tpl",
            "_ingress.tpl",
            "_helpers.tpl",
        ],
    )
    def test_common_templates_exist(self, template: str) -> None:
        path = HELM / "charts" / "k1s0-common" / "templates" / template
        assert path.exists(), f"k1s0-common/templates/{template} が存在しません"


class TestHelmServiceCharts:
    """helm設計.md: サービス別 Chart の検証。"""

    @pytest.mark.parametrize(
        "service_path",
        [
            "system/auth",
            "system/kong",
            "business/accounting/ledger",
            "service/order",
        ],
    )
    def test_service_chart_exists(self, service_path: str) -> None:
        chart = HELM / "services" / service_path / "Chart.yaml"
        assert chart.exists(), f"services/{service_path}/Chart.yaml が存在しません"

    @pytest.mark.parametrize(
        "service_path",
        [
            "system/auth",
            "system/kong",
            "business/accounting/ledger",
            "service/order",
        ],
    )
    def test_service_values_files(self, service_path: str) -> None:
        base = HELM / "services" / service_path
        assert (base / "values.yaml").exists(), f"{service_path}/values.yaml がありません"
        assert (base / "values-dev.yaml").exists(), f"{service_path}/values-dev.yaml がありません"
        assert (base / "values-staging.yaml").exists(), f"{service_path}/values-staging.yaml がありません"
        assert (base / "values-prod.yaml").exists(), f"{service_path}/values-prod.yaml がありません"

    @pytest.mark.parametrize(
        "service_path",
        [
            "system/auth",
            "system/kong",
            "business/accounting/ledger",
            "service/order",
        ],
    )
    def test_service_templates_dir(self, service_path: str) -> None:
        templates = HELM / "services" / service_path / "templates"
        assert templates.exists(), f"{service_path}/templates/ がありません"

    def test_chart_depends_on_common(self) -> None:
        """各サービスChartがk1s0-commonに依存していること。"""
        chart_path = HELM / "services" / "service" / "order" / "Chart.yaml"
        with open(chart_path, encoding="utf-8") as f:
            chart = yaml.safe_load(f)
        assert "dependencies" in chart
        dep_names = [d["name"] for d in chart["dependencies"]]
        assert "k1s0-common" in dep_names


class TestHelmTierDirectories:
    """helm設計.md: Tier別ディレクトリの検証。"""

    @pytest.mark.parametrize("tier", ["system", "business", "service"])
    def test_tier_directory_exists(self, tier: str) -> None:
        assert (HELM / "services" / tier).exists(), f"services/{tier}/ が存在しません"


class TestHelmOrderValues:
    """helm設計.md: order サービスの values.yaml 設定値検証。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "values.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_image_registry(self) -> None:
        assert self.values["image"]["registry"] == "harbor.internal.example.com"

    def test_image_pull_policy(self) -> None:
        assert self.values["image"]["pullPolicy"] == "IfNotPresent"

    def test_container_port(self) -> None:
        assert self.values["container"]["port"] == 8080

    def test_service_type(self) -> None:
        assert self.values["service"]["type"] == "ClusterIP"

    def test_service_port(self) -> None:
        assert self.values["service"]["port"] == 80

    def test_autoscaling(self) -> None:
        assert self.values["autoscaling"]["enabled"] is True
        assert self.values["autoscaling"]["minReplicas"] == 2
        assert self.values["autoscaling"]["maxReplicas"] == 5
        assert self.values["autoscaling"]["targetCPUUtilizationPercentage"] == 70

    def test_pdb(self) -> None:
        assert self.values["pdb"]["enabled"] is True
        assert self.values["pdb"]["minAvailable"] == 1

    def test_security_context(self) -> None:
        assert self.values["podSecurityContext"]["runAsNonRoot"] is True
        assert self.values["podSecurityContext"]["runAsUser"] == 65532

    def test_container_security_context(self) -> None:
        ctx = self.values["containerSecurityContext"]
        assert ctx["readOnlyRootFilesystem"] is True
        assert ctx["allowPrivilegeEscalation"] is False

    def test_vault(self) -> None:
        assert self.values["vault"]["enabled"] is True
        assert self.values["vault"]["role"] == "service"

    def test_probes(self) -> None:
        assert self.values["probes"]["liveness"]["httpGet"]["path"] == "/healthz"
        assert self.values["probes"]["readiness"]["httpGet"]["path"] == "/readyz"

    def test_tier_label(self) -> None:
        assert self.values["labels"]["tier"] == "service"


class TestHelmOrderDevValues:
    """helm設計.md: order values-dev.yaml 検証。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "values-dev.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_replica_count(self) -> None:
        assert self.values["replicaCount"] == 1

    def test_autoscaling_disabled(self) -> None:
        assert self.values["autoscaling"]["enabled"] is False

    def test_pdb_disabled(self) -> None:
        assert self.values["pdb"]["enabled"] is False

    def test_vault_disabled(self) -> None:
        assert self.values["vault"]["enabled"] is False


class TestHelmStagingValues:
    """helm設計.md: order values-staging.yaml 設定値検証。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "values-staging.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_replica_count(self) -> None:
        assert self.values["replicaCount"] == 2

    def test_autoscaling_min_replicas(self) -> None:
        assert self.values["autoscaling"]["minReplicas"] == 2

    def test_autoscaling_max_replicas(self) -> None:
        assert self.values["autoscaling"]["maxReplicas"] == 5


class TestHelmProdValues:
    """helm設計.md: order values-prod.yaml 設定値検証。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "values-prod.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_replica_count(self) -> None:
        assert self.values["replicaCount"] == 3

    def test_autoscaling_min_replicas(self) -> None:
        assert self.values["autoscaling"]["minReplicas"] == 3

    def test_autoscaling_max_replicas(self) -> None:
        assert self.values["autoscaling"]["maxReplicas"] == 10

    def test_pod_anti_affinity(self) -> None:
        """helm設計.md: prod では podAntiAffinity が設定されていること。"""
        affinity = self.values["affinity"]
        assert "podAntiAffinity" in affinity
        rules = affinity["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"]
        assert len(rules) > 0
        assert rules[0]["topologyKey"] == "kubernetes.io/hostname"


class TestHelmChartYaml:
    """helm設計.md: Chart.yaml apiVersion/type/dependencies テスト。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "Chart.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.chart = yaml.safe_load(f)

    def test_api_version(self) -> None:
        assert self.chart["apiVersion"] == "v2"

    def test_chart_type(self) -> None:
        assert self.chart["type"] == "application"

    def test_dependencies(self) -> None:
        assert "dependencies" in self.chart
        dep_names = [d["name"] for d in self.chart["dependencies"]]
        assert "k1s0-common" in dep_names

    def test_dependency_repository(self) -> None:
        """helm設計.md: 依存の repository が相対パスであること。"""
        dep = [d for d in self.chart["dependencies"] if d["name"] == "k1s0-common"][0]
        assert dep["repository"].startswith("file://")


class TestHelmVaultAnnotations:
    """helm設計.md: Vault Agent Injector annotations テスト。"""

    def test_vault_enabled_in_values(self) -> None:
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["vault"]["enabled"] is True
        assert values["vault"]["role"] == "service"

    def test_vault_secrets_config(self) -> None:
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        secrets = values["vault"]["secrets"]
        assert len(secrets) > 0
        assert "secret/data/k1s0/service/order/database" in secrets[0]["path"]


class TestHelmKafkaRedisConfig:
    """helm設計.md: kafka/redis 設定テスト。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_kafka_config(self) -> None:
        assert "kafka" in self.values
        assert self.values["kafka"]["enabled"] is False
        assert self.values["kafka"]["brokers"] == []

    def test_redis_config(self) -> None:
        assert "redis" in self.values
        assert self.values["redis"]["enabled"] is False
        assert self.values["redis"]["host"] == ""


class TestHelmLibraryChartVersioning:
    """helm設計.md: Library Chart バージョニング方針テスト。"""

    def test_chart_version_exists(self) -> None:
        """helm設計.md: k1s0-common Chart.yaml に version が定義されていること。"""
        path = HELM / "charts" / "k1s0-common" / "Chart.yaml"
        with open(path, encoding="utf-8") as f:
            chart = yaml.safe_load(f)
        assert "version" in chart
        # SemVer 形式: X.Y.Z
        parts = chart["version"].split(".")
        assert len(parts) == 3, f"version が SemVer 形式でありません: {chart['version']}"


class TestHelmDevResourceValues:
    """helm設計.md: values-dev.yaml リソース値テスト。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "values-dev.yaml"
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_dev_resource_requests_cpu(self) -> None:
        assert str(self.values["resources"]["requests"]["cpu"]) == "100m"

    def test_dev_resource_requests_memory(self) -> None:
        assert str(self.values["resources"]["requests"]["memory"]) == "128Mi"

    def test_dev_resource_limits_cpu(self) -> None:
        assert str(self.values["resources"]["limits"]["cpu"]) == "500m"

    def test_dev_resource_limits_memory(self) -> None:
        assert str(self.values["resources"]["limits"]["memory"]) == "512Mi"

    def test_dev_config_data(self) -> None:
        """helm設計.md: values-dev.yaml の config.data に config.yaml が含まれること。"""
        config_data = self.values["config"]["data"]
        assert "config.yaml" in config_data
        assert "dev" in config_data["config.yaml"]


class TestHelmStagingResourceValues:
    """helm設計.md: values-staging.yaml リソース値テスト。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "values-staging.yaml"
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_staging_resource_requests_cpu(self) -> None:
        assert str(self.values["resources"]["requests"]["cpu"]) == "250m"

    def test_staging_resource_requests_memory(self) -> None:
        assert str(self.values["resources"]["requests"]["memory"]) == "256Mi"

    def test_staging_resource_limits_cpu(self) -> None:
        assert str(self.values["resources"]["limits"]["cpu"]) == "1000m"

    def test_staging_resource_limits_memory(self) -> None:
        assert str(self.values["resources"]["limits"]["memory"]) == "1Gi"

    def test_staging_config_data(self) -> None:
        """helm設計.md: values-staging.yaml の config.data に config.yaml が含まれること。"""
        config_data = self.values["config"]["data"]
        assert "config.yaml" in config_data
        assert "staging" in config_data["config.yaml"]


class TestHelmProdResourceValues:
    """helm設計.md: values-prod.yaml リソース値テスト。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "service" / "order" / "values-prod.yaml"
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_prod_resource_requests_cpu(self) -> None:
        assert str(self.values["resources"]["requests"]["cpu"]) == "500m"

    def test_prod_resource_requests_memory(self) -> None:
        assert str(self.values["resources"]["requests"]["memory"]) == "512Mi"

    def test_prod_resource_limits_cpu(self) -> None:
        assert str(self.values["resources"]["limits"]["cpu"]) == "2000m"

    def test_prod_resource_limits_memory(self) -> None:
        assert str(self.values["resources"]["limits"]["memory"]) == "2Gi"

    def test_prod_config_data(self) -> None:
        """helm設計.md: values-prod.yaml の config.data に config.yaml が含まれること。"""
        config_data = self.values["config"]["data"]
        assert "config.yaml" in config_data
        assert "prod" in config_data["config.yaml"]


class TestHelmDeploymentTemplate:
    """helm設計.md: _deployment.tpl 内容詳細テスト。"""

    def setup_method(self) -> None:
        path = HELM / "charts" / "k1s0-common" / "templates" / "_deployment.tpl"
        self.content = path.read_text(encoding="utf-8")

    def test_deployment_kind(self) -> None:
        assert "kind: Deployment" in self.content

    def test_deployment_autoscaling_check(self) -> None:
        """helm設計.md: autoscaling.enabled チェックが含まれること。"""
        assert ".Values.autoscaling.enabled" in self.content

    def test_deployment_image_template(self) -> None:
        """helm設計.md: イメージテンプレートが registry/repository:tag 形式であること。"""
        assert ".Values.image.registry" in self.content
        assert ".Values.image.repository" in self.content
        assert ".Values.image.tag" in self.content

    def test_deployment_grpc_port(self) -> None:
        """helm設計.md: grpcPort 条件分岐が含まれること。"""
        assert ".Values.container.grpcPort" in self.content

    def test_deployment_security_context(self) -> None:
        """helm設計.md: securityContext が含まれること。"""
        assert ".Values.podSecurityContext" in self.content
        assert ".Values.containerSecurityContext" in self.content

    def test_deployment_config_volume(self) -> None:
        """helm設計.md: config ボリュームがマウントされること。"""
        assert "config" in self.content
        assert ".Values.config.mountPath" in self.content

    def test_deployment_image_pull_secrets(self) -> None:
        """helm設計.md: imagePullSecrets が含まれること。"""
        assert ".Values.imagePullSecrets" in self.content

    def test_deployment_service_account(self) -> None:
        """helm設計.md: serviceAccountName が含まれること。"""
        assert "serviceAccountName" in self.content


class TestHelmServiceTemplate:
    """helm設計.md: _service.tpl 内容詳細テスト。"""

    def setup_method(self) -> None:
        path = HELM / "charts" / "k1s0-common" / "templates" / "_service.tpl"
        self.content = path.read_text(encoding="utf-8")

    def test_service_kind(self) -> None:
        assert "kind: Service" in self.content

    def test_service_type(self) -> None:
        assert ".Values.service.type" in self.content

    def test_service_http_port(self) -> None:
        assert "name: http" in self.content

    def test_service_grpc_port(self) -> None:
        """helm設計.md: gRPC ポートが条件分岐で含まれること。"""
        assert ".Values.service.grpcPort" in self.content
        assert "name: grpc" in self.content

    def test_service_selector_labels(self) -> None:
        assert "k1s0-common.selectorLabels" in self.content


class TestHelmIngressTemplate:
    """helm設計.md: _ingress.tpl ルーティング方針テスト。"""

    def setup_method(self) -> None:
        path = HELM / "charts" / "k1s0-common" / "templates" / "_ingress.tpl"
        self.content = path.read_text(encoding="utf-8")

    def test_ingress_enabled_check(self) -> None:
        """helm設計.md: ingress.enabled チェックが含まれること。"""
        assert ".Values.ingress.enabled" in self.content

    def test_ingress_class_name_default_nginx(self) -> None:
        """helm設計.md: ingressClassName のデフォルトが nginx であること。"""
        assert '.Values.ingress.ingressClassName | default "nginx"' in self.content

    def test_ingress_tls_support(self) -> None:
        assert ".Values.ingress.tls" in self.content

    def test_ingress_hosts_range(self) -> None:
        assert ".Values.ingress.hosts" in self.content


class TestHelmDeployCommandFormat:
    """helm設計.md: デプロイコマンド形式テスト。"""

    def test_values_yaml_supports_image_tag_override(self) -> None:
        """helm設計.md: image.tag が空文字列でCI/CD上書き可能であること。"""
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["image"]["tag"] == ""


class TestHelmImagePullSecrets:
    """helm設計.md: imagePullSecrets テスト。"""

    def test_image_pull_secrets_defined(self) -> None:
        """helm設計.md: imagePullSecrets が harbor-pull-secret であること。"""
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert "imagePullSecrets" in values
        names = [s["name"] for s in values["imagePullSecrets"]]
        assert "harbor-pull-secret" in names


class TestHelmGrpcPortDefault:
    """helm設計.md: container.grpcPort デフォルト値テスト。"""

    def test_grpc_port_default_null(self) -> None:
        """helm設計.md: container.grpcPort のデフォルトが null であること。"""
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["container"]["grpcPort"] is None

    def test_service_grpc_port_default_null(self) -> None:
        """helm設計.md: service.grpcPort のデフォルトが null であること。"""
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["service"]["grpcPort"] is None


class TestHelmServiceAccount:
    """helm設計.md: serviceAccount 設定テスト。"""

    def test_service_account_create(self) -> None:
        """helm設計.md: serviceAccount.create がデフォルト true であること。"""
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["serviceAccount"]["create"] is True

    def test_service_account_name_empty(self) -> None:
        """helm設計.md: serviceAccount.name がデフォルト空文字列であること。"""
        path = HELM / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["serviceAccount"]["name"] == ""


class TestHelmKongValues:
    """APIゲートウェイ設計.md: Kong values.yaml 検証。"""

    def setup_method(self) -> None:
        path = HELM / "services" / "system" / "kong" / "values.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.values = yaml.safe_load(f)

    def test_kong_image_tag(self) -> None:
        assert self.values["image"]["tag"] == "3.7"

    def test_database_postgres(self) -> None:
        assert self.values["env"]["database"] == "postgres"

    def test_proxy_type(self) -> None:
        assert self.values["proxy"]["type"] == "ClusterIP"

    def test_ingress_controller_disabled(self) -> None:
        assert self.values["ingressController"]["enabled"] is False

    def test_external_postgresql(self) -> None:
        assert self.values["postgresql"]["enabled"] is False

    def test_replica_count(self) -> None:
        assert self.values["replicaCount"] == 2
