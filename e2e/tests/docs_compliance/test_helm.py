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
