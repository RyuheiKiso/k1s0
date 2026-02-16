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
            "k1s0-system.yaml",
            "k1s0-business.yaml",
            "k1s0-service.yaml",
            "observability.yaml",
            "messaging.yaml",
            "ingress.yaml",
            "service-mesh.yaml",
            "cert-manager.yaml",
            "harbor.yaml",
        ],
    )
    def test_namespace_file_exists(self, namespace_file: str) -> None:
        path = K8S / "namespaces" / namespace_file
        assert path.exists(), f"namespaces/{namespace_file} が存在しません"


class TestKubernetesResourceQuota:
    """kubernetes設計.md: ResourceQuota 値の検証。"""

    @pytest.mark.parametrize(
        "ns_file,expected",
        [
            ("k1s0-system.yaml", {
                "requests.cpu": "8",
                "requests.memory": "16Gi",
                "limits.cpu": "16",
                "limits.memory": "32Gi",
                "pods": "50",
            }),
            ("k1s0-business.yaml", {
                "requests.cpu": "16",
                "requests.memory": "32Gi",
                "limits.cpu": "32",
                "limits.memory": "64Gi",
                "pods": "100",
            }),
            ("k1s0-service.yaml", {
                "requests.cpu": "8",
                "requests.memory": "16Gi",
                "limits.cpu": "16",
                "limits.memory": "32Gi",
                "pods": "50",
            }),
        ],
    )
    def test_resource_quota_values(self, ns_file: str, expected: dict) -> None:
        """kubernetes設計.md: ResourceQuota の数値が仕様通りであること。"""
        path = K8S / "namespaces" / ns_file
        docs = list(yaml.safe_load_all(path.read_text(encoding="utf-8")))
        quotas = [d for d in docs if d and d.get("kind") == "ResourceQuota"]
        assert len(quotas) > 0, f"{ns_file} に ResourceQuota が定義されていません"
        hard = quotas[0]["spec"]["hard"]
        for key, val in expected.items():
            assert str(hard[key]) == val, f"{ns_file}: {key} が {val} ではなく {hard[key]}"


class TestKubernetesLimitRange:
    """kubernetes設計.md: LimitRange 設定の検証。"""

    @pytest.mark.parametrize(
        "ns_file",
        ["k1s0-system.yaml", "k1s0-business.yaml", "k1s0-service.yaml"],
    )
    def test_limit_range_exists(self, ns_file: str) -> None:
        """kubernetes設計.md: 各 Namespace に LimitRange が定義されていること。"""
        path = K8S / "namespaces" / ns_file
        docs = list(yaml.safe_load_all(path.read_text(encoding="utf-8")))
        lr = [d for d in docs if d and d.get("kind") == "LimitRange"]
        assert len(lr) > 0, f"{ns_file} に LimitRange が定義されていません"

    def test_limit_range_defaults(self) -> None:
        """kubernetes設計.md: LimitRange のデフォルト値が仕様通りであること。"""
        path = K8S / "namespaces" / "k1s0-service.yaml"
        docs = list(yaml.safe_load_all(path.read_text(encoding="utf-8")))
        lr = [d for d in docs if d and d.get("kind") == "LimitRange"][0]
        limits = lr["spec"]["limits"][0]
        assert limits["default"]["cpu"] == "1"
        assert limits["default"]["memory"] == "1Gi"
        assert limits["defaultRequest"]["cpu"] == "250m"
        assert limits["defaultRequest"]["memory"] == "256Mi"
        assert limits["type"] == "Container"


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

    def _get_role(self, name: str) -> dict:
        for doc in self.docs:
            if doc and doc.get("kind") == "ClusterRole" and doc["metadata"]["name"] == name:
                return doc
        raise AssertionError(f"ClusterRole '{name}' が見つかりません")

    def test_developer_rules(self) -> None:
        """kubernetes設計.md: k1s0-developer の rules が仕様通りであること。"""
        role = self._get_role("k1s0-developer")
        rules = role["rules"]
        # ルール1: core API pods, services, configmaps -> get, list, watch
        core_rule = [r for r in rules if "" in r["apiGroups"]]
        assert len(core_rule) > 0
        assert set(core_rule[0]["resources"]) == {"pods", "services", "configmaps"}
        assert set(core_rule[0]["verbs"]) == {"get", "list", "watch"}
        # ルール2: apps API deployments -> get, list, watch
        apps_rule = [r for r in rules if "apps" in r["apiGroups"]]
        assert len(apps_rule) > 0
        assert "deployments" in apps_rule[0]["resources"]
        assert set(apps_rule[0]["verbs"]) == {"get", "list", "watch"}

    def test_operator_rules(self) -> None:
        """kubernetes設計.md: k1s0-operator の rules が仕様通りであること。"""
        role = self._get_role("k1s0-operator")
        rules = role["rules"]
        core_rule = [r for r in rules if "" in r["apiGroups"]]
        assert len(core_rule) > 0
        assert "secrets" in core_rule[0]["resources"]
        assert "create" in core_rule[0]["verbs"]
        assert "delete" in core_rule[0]["verbs"]
        apps_rule = [r for r in rules if "apps" in r["apiGroups"]]
        assert len(apps_rule) > 0
        assert "statefulsets" in apps_rule[0]["resources"]
        assert "patch" in apps_rule[0]["verbs"]

    def test_admin_rules(self) -> None:
        """kubernetes設計.md: k1s0-admin はクラスタ全体の管理権限を持つこと。"""
        role = self._get_role("k1s0-admin")
        rules = role["rules"]
        assert rules[0]["apiGroups"] == ["*"]
        assert rules[0]["resources"] == ["*"]
        assert rules[0]["verbs"] == ["*"]

    def test_readonly_rules(self) -> None:
        """kubernetes設計.md: readonly は全リソースの参照のみであること。"""
        role = self._get_role("readonly")
        rules = role["rules"]
        assert rules[0]["apiGroups"] == ["*"]
        assert rules[0]["resources"] == ["*"]
        assert set(rules[0]["verbs"]) == {"get", "list", "watch"}


class TestKubernetesIngress:
    """kubernetes設計.md: Ingress リソースの検証。"""

    def setup_method(self) -> None:
        path = K8S / "ingress" / "ingress.yaml"
        assert path.exists(), "ingress/ingress.yaml が存在しません"
        self.docs = list(yaml.safe_load_all(path.read_text(encoding="utf-8")))

    def test_ingress_directory_exists(self) -> None:
        assert (K8S / "ingress").exists(), "kubernetes/ingress が存在しません"

    def test_ingress_annotations(self) -> None:
        """kubernetes設計.md: Ingress annotations が仕様通りであること。"""
        ingress = [d for d in self.docs if d and d.get("kind") == "Ingress"][0]
        annotations = ingress["metadata"]["annotations"]
        assert annotations["nginx.ingress.kubernetes.io/ssl-redirect"] == "true"
        assert annotations["cert-manager.io/cluster-issuer"] == "internal-ca"

    def test_ingress_tls(self) -> None:
        """kubernetes設計.md: Ingress TLS 設定が仕様通りであること。"""
        ingress = [d for d in self.docs if d and d.get("kind") == "Ingress"][0]
        tls = ingress["spec"]["tls"]
        assert len(tls) > 0
        assert "*.k1s0.internal.example.com" in tls[0]["hosts"]
        assert tls[0]["secretName"] == "k1s0-tls"

    def test_ingress_class_name(self) -> None:
        """kubernetes設計.md: ingressClassName が nginx であること。"""
        ingress = [d for d in self.docs if d and d.get("kind") == "Ingress"][0]
        assert ingress["spec"]["ingressClassName"] == "nginx"


class TestKubernetesStorageClass:
    """kubernetes設計.md: StorageClass 用途テスト。"""

    def setup_method(self) -> None:
        path = ROOT / "infra" / "terraform" / "modules" / "kubernetes-storage" / "main.tf"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "sc_name,usage",
        [
            ("ceph-block", "一般用途"),
            ("ceph-filesystem", "共有ストレージ"),
            ("ceph-block-fast", "データベース用"),
        ],
    )
    def test_storage_class_defined(self, sc_name: str, usage: str) -> None:
        """kubernetes設計.md: 3 種類の StorageClass が定義されていること。"""
        assert sc_name in self.content, f"StorageClass '{sc_name}'({usage}) が定義されていません"


class TestKubernetesLabels:
    """kubernetes設計.md: ラベル規約テスト。"""

    def setup_method(self) -> None:
        helpers = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates" / "_helpers.tpl"
        assert helpers.exists(), "_helpers.tpl が存在しません"
        self.helpers_content = helpers.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "label",
        [
            "app.kubernetes.io/name",
            "app.kubernetes.io/version",
            "app.kubernetes.io/component",
            "app.kubernetes.io/part-of",
            "app.kubernetes.io/managed-by",
            "tier",
        ],
    )
    def test_label_in_helm_helpers(self, label: str) -> None:
        """kubernetes設計.md: 6 標準ラベルが Helm テンプレートまたは設定ファイルで使用されていること。"""
        if label == "tier":
            # tier は values.yaml の labels セクションで動的に設定される
            # _helpers.tpl では .Values.labels をイテレートして出力する
            assert ".Values.labels" in self.helpers_content, (
                "ラベル 'tier' の出力処理（.Values.labels のイテレーション）が _helpers.tpl に存在しません"
            )
        else:
            assert label in self.helpers_content, f"ラベル '{label}' が _helpers.tpl に存在しません"


class TestKubernetesNetworkPolicies:
    """kubernetes設計.md: NetworkPolicy が Terraform で定義されていることの検証。"""

    def setup_method(self) -> None:
        tf_path = ROOT / "infra" / "terraform" / "modules" / "kubernetes-base" / "main.tf"
        assert tf_path.exists()
        self.content = tf_path.read_text(encoding="utf-8")

    def test_network_policy_in_terraform(self) -> None:
        """kubernetes設計.md: NetworkPolicy は terraform kubernetes-base モジュールで管理。"""
        assert "kubernetes_network_policy" in self.content

    def test_network_policy_deny_cross_tier(self) -> None:
        """kubernetes設計.md: deny-cross-tier ポリシーが定義されていること。"""
        assert "deny-cross-tier" in self.content or "deny_cross_tier" in self.content

    def test_network_policy_ingress_rules(self) -> None:
        """kubernetes設計.md: ingress ルールが定義されていること。"""
        assert "ingress" in self.content.lower()
        assert "namespace_selector" in self.content


class TestKubernetesHPABehavior:
    """kubernetes設計.md: HPA behavior 設定テスト。"""

    def test_hpa_template_exists(self) -> None:
        """kubernetes設計.md: HPA テンプレートが存在すること。"""
        path = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates" / "_hpa.tpl"
        assert path.exists(), "_hpa.tpl が存在しません"


class TestKubernetesEnvironmentHPA:
    """kubernetes設計.md: 環境別 HPA 設定テスト。"""

    def test_dev_hpa_disabled(self) -> None:
        """kubernetes設計.md: dev では HPA 無効がデフォルト。"""
        path = ROOT / "infra" / "helm" / "services" / "service" / "order" / "values-dev.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["autoscaling"]["enabled"] is False

    def test_staging_hpa_replicas(self) -> None:
        """kubernetes設計.md: staging は minReplicas:2, maxReplicas:5。"""
        path = ROOT / "infra" / "helm" / "services" / "service" / "order" / "values-staging.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["autoscaling"]["minReplicas"] == 2
        assert values["autoscaling"]["maxReplicas"] == 5

    def test_prod_hpa_replicas(self) -> None:
        """kubernetes設計.md: prod は minReplicas:3, maxReplicas:10。"""
        path = ROOT / "infra" / "helm" / "services" / "service" / "order" / "values-prod.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["autoscaling"]["minReplicas"] == 3
        assert values["autoscaling"]["maxReplicas"] == 10


class TestKubernetesPDB:
    """kubernetes設計.md: PodDisruptionBudget テスト。"""

    def test_pdb_min_available(self) -> None:
        """kubernetes設計.md: PDB minAvailable が 1 であること。"""
        path = ROOT / "infra" / "helm" / "services" / "service" / "order" / "values.yaml"
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        assert values["pdb"]["minAvailable"] == 1

    def test_pdb_template_exists(self) -> None:
        """kubernetes設計.md: PDB テンプレートが存在すること。"""
        path = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates" / "_pdb.tpl"
        assert path.exists(), "_pdb.tpl が存在しません"
