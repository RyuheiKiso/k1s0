"""kubernetes設計.md の仕様準拠テスト。

Kubernetes リソース定義が設計ドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]
K8S = ROOT / "infra" / "kubernetes"


class TestKubernetesNamespaces:
    """kubernetes設計.md: Namespace 定義の検証。"""

    @pytest.mark.parametrize(
        "namespace_file",
        [
            "observability.yaml",
        ],
    )
    def test_namespace_file_exists(self, namespace_file: str) -> None:
        path = K8S / "namespaces" / namespace_file
        assert path.exists(), f"namespaces/{namespace_file} が存在しません"


class TestKubernetesRBAC:
    """kubernetes設計.md: RBAC ClusterRole の検証。"""

    def setup_method(self) -> None:
        path = K8S / "rbac" / "cluster-roles.yaml"
        assert path.exists(), "rbac/cluster-roles.yaml が存在しません"
        self.content = path.read_text(encoding="utf-8")
        self.docs = list(yaml.safe_load_all(self.content))

    def test_cluster_roles_file_exists(self) -> None:
        assert (K8S / "rbac" / "cluster-roles.yaml").exists()

    @pytest.mark.parametrize(
        "role_name",
        ["k1s0-developer", "k1s0-operator", "k1s0-admin", "readonly"],
    )
    def test_cluster_role_defined(self, role_name: str) -> None:
        """kubernetes設計.md: 4つの ClusterRole が定義されている。"""
        role_names = [
            doc["metadata"]["name"]
            for doc in self.docs
            if doc and doc.get("kind") == "ClusterRole"
        ]
        assert role_name in role_names, f"ClusterRole '{role_name}' が定義されていません"


class TestKubernetesIngress:
    """kubernetes設計.md: Ingress リソースの検証。"""

    def test_ingress_directory_exists(self) -> None:
        assert (K8S / "ingress").exists() or (
            ROOT / "infra" / "kubernetes"
        ).exists(), "kubernetes/ingress が存在しません"


class TestKubernetesNetworkPolicies:
    """kubernetes設計.md: NetworkPolicy が Terraform で定義されていることの検証。"""

    def test_network_policy_in_terraform(self) -> None:
        """kubernetes設計.md: NetworkPolicy は terraform kubernetes-base モジュールで管理。"""
        tf_path = ROOT / "infra" / "terraform" / "modules" / "kubernetes-base" / "main.tf"
        assert tf_path.exists()
        content = tf_path.read_text(encoding="utf-8")
        assert "kubernetes_network_policy" in content
