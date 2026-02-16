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


class TestInfraEnvironmentNodes:
    """インフラ設計.md: 環境構成ノード台数テスト。"""

    def test_environment_nodes_in_doc(self) -> None:
        """インフラ設計.md: 環境構成テーブル（dev/staging/prod）が記載されていること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert "dev" in doc
        assert "staging" in doc
        assert "prod" in doc
        # Master / Worker 台数
        assert "Master" in doc
        assert "Worker" in doc


class TestInfraServerRequirements:
    """インフラ設計.md: サーバー要件テスト。"""

    def test_server_requirements_in_doc(self) -> None:
        """インフラ設計.md: Master/Worker ノードのサーバー要件が記載されていること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert "vCPU" in doc
        assert "GB" in doc or "SSD" in doc


class TestInfraNetworkDesign:
    """インフラ設計.md: ネットワーク設計 4 CIDR テスト。"""

    def test_network_4_cidrs(self) -> None:
        """インフラ設計.md: 4 つの CIDR セグメントが記載されていること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert "10.244.0.0/16" in doc     # Pod ネットワーク
        assert "10.96.0.0/12" in doc      # Service ネットワーク
        assert "10.0.100.0/24" in doc     # MetalLB プール
        assert "10.0.200.0/24" in doc     # 管理ネットワーク


class TestInfraStorageCeph:
    """インフラ設計.md: ストレージ構成 (Ceph) テスト。"""

    def test_ceph_storage_types(self) -> None:
        """インフラ設計.md: 3 種類のストレージ（ブロック/ファイル/オブジェクト）が記載されていること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert "Ceph RBD" in doc
        assert "CephFS" in doc
        assert "Ceph RGW" in doc


class TestInfraHarborSettings:
    """インフラ設計.md: Harbor 設定テスト。"""

    def test_harbor_url(self) -> None:
        """インフラ設計.md: Harbor URL が harbor.internal.example.com であること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert "harbor.internal.example.com" in doc

    def test_harbor_trivy(self) -> None:
        """インフラ設計.md: Harbor の脆弱性スキャンに Trivy が使用されること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert "Trivy" in doc


class TestInfraK8sClusterComponents:
    """インフラ設計.md: K8s クラスタ構築コンポーネントテスト。"""

    @pytest.mark.parametrize(
        "component",
        ["kubeadm", "Calico", "Nginx Ingress", "MetalLB", "Ceph CSI", "CoreDNS", "cert-manager", "Flagger"],
    )
    def test_cluster_component_documented(self, component: str) -> None:
        """インフラ設計.md: K8s クラスタ構築コンポーネントが記載されていること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert component in doc, f"コンポーネント '{component}' が記載されていません"


class TestInfraConfigManagementTools:
    """インフラ設計.md: 構成管理ツール責務分担テスト。"""

    @pytest.mark.parametrize(
        "tool,usage",
        [
            ("Ansible", "OS"),
            ("Terraform", "Namespace"),
            ("Helm", "デプロイ"),
        ],
    )
    def test_tool_responsibility(self, tool: str, usage: str) -> None:
        """インフラ設計.md: 構成管理ツールの責務分担が記載されていること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert tool in doc
        assert usage in doc


class TestInfraBackupSettings:
    """インフラ設計.md: バックアップ設定テスト。"""

    @pytest.mark.parametrize(
        "target",
        ["etcd", "データベース", "Ceph", "Harbor", "Vault"],
    )
    def test_backup_target_documented(self, target: str) -> None:
        """インフラ設計.md: バックアップ対象が記載されていること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert target in doc


class TestInfraStorageClassYAML:
    """インフラ設計.md: StorageClass YAML 内容テスト。"""

    def test_storage_class_yaml_in_doc(self) -> None:
        """インフラ設計.md: StorageClass YAML の provisioner が rbd.csi.ceph.com であること。"""
        doc = (ROOT / "docs" / "インフラ設計.md").read_text(encoding="utf-8")
        assert "rbd.csi.ceph.com" in doc
        assert "ceph-block" in doc
        assert "allowVolumeExpansion" in doc
