"""インフラ設計関連ドキュメントの仕様準拠テスト。

APIゲートウェイ設計.md, サービスメッシュ設計.md, 可観測性設計.md,
メッセージング設計.md, Dockerイメージ戦略.md の仕様と実装の一致を検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]


class TestKongConfig:
    """APIゲートウェイ設計.md: Kong 設定の検証。"""

    def test_kong_yaml_exists(self) -> None:
        assert (ROOT / "infra" / "kong" / "kong.yaml").exists()

    def test_global_plugins_exists(self) -> None:
        assert (ROOT / "infra" / "kong" / "plugins" / "global.yaml").exists()

    def test_auth_plugins_exists(self) -> None:
        assert (ROOT / "infra" / "kong" / "plugins" / "auth.yaml").exists()

    @pytest.mark.parametrize("tier", ["system", "business", "service"])
    def test_tier_service_config(self, tier: str) -> None:
        path = ROOT / "infra" / "kong" / "services" / f"{tier}.yaml"
        assert path.exists(), f"kong/services/{tier}.yaml が存在しません"


class TestIstioConfig:
    """サービスメッシュ設計.md: Istio 設定の検証。"""

    def test_gateway_exists(self) -> None:
        assert (ROOT / "infra" / "istio" / "gateway.yaml").exists()

    def test_virtual_service_exists(self) -> None:
        assert (ROOT / "infra" / "istio" / "virtual-service.yaml").exists()

    def test_gateway_content(self) -> None:
        content = (ROOT / "infra" / "istio" / "gateway.yaml").read_text(encoding="utf-8")
        assert "Gateway" in content

    def test_virtual_service_content(self) -> None:
        content = (ROOT / "infra" / "istio" / "virtual-service.yaml").read_text(encoding="utf-8")
        assert "VirtualService" in content


class TestObservabilityInfra:
    """可観測性設計.md: 監視インフラの検証。"""

    def test_prometheus_config_exists(self) -> None:
        assert (ROOT / "infra" / "docker" / "prometheus" / "prometheus.yaml").exists()

    def test_grafana_provisioning_exists(self) -> None:
        assert (ROOT / "infra" / "docker" / "grafana" / "provisioning").exists()

    def test_grafana_dashboards_exists(self) -> None:
        assert (ROOT / "infra" / "docker" / "grafana" / "dashboards").exists()

    def test_observability_terraform_module(self) -> None:
        module = ROOT / "infra" / "terraform" / "modules" / "observability"
        assert module.exists()
        assert (module / "main.tf").exists()


class TestMessagingInfra:
    """メッセージング設計.md: Kafka関連設定の検証。"""

    def test_messaging_terraform_module(self) -> None:
        module = ROOT / "infra" / "terraform" / "modules" / "messaging"
        assert module.exists()
        assert (module / "main.tf").exists()


class TestDockerImages:
    """Dockerイメージ戦略.md: Docker関連の検証。"""

    def test_base_images_dir(self) -> None:
        assert (ROOT / "infra" / "docker" / "base-images").exists()

    def test_init_db_dir(self) -> None:
        assert (ROOT / "infra" / "docker" / "init-db").exists()

    def test_keycloak_dir(self) -> None:
        """docker-compose設計.md で keycloak の realm import 用ディレクトリ。"""
        assert (ROOT / "infra" / "docker" / "keycloak").exists()


class TestVaultConfig:
    """認証認可設計.md: Vault 設定の検証。"""

    def test_vault_terraform_module(self) -> None:
        module = ROOT / "infra" / "terraform" / "modules" / "vault"
        assert module.exists()
        assert (module / "main.tf").exists()


class TestHarborConfig:
    """terraform設計.md: Harbor 設定の検証。"""

    def test_harbor_terraform_module(self) -> None:
        module = ROOT / "infra" / "terraform" / "modules" / "harbor"
        assert module.exists()
        assert (module / "main.tf").exists()


class TestDatabaseTerraform:
    """terraform設計.md: Database モジュールの検証。"""

    def test_database_terraform_module(self) -> None:
        module = ROOT / "infra" / "terraform" / "modules" / "database"
        assert module.exists()
        assert (module / "main.tf").exists()


class TestServiceMeshTerraform:
    """サービスメッシュ設計.md: Service Mesh モジュールの検証。"""

    def test_service_mesh_terraform_module(self) -> None:
        module = ROOT / "infra" / "terraform" / "modules" / "service-mesh"
        assert module.exists()
        assert (module / "main.tf").exists()
