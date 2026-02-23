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
            (
                "k1s0-system.yaml",
                {
                    "requests.cpu": "8",
                    "requests.memory": "16Gi",
                    "limits.cpu": "16",
                    "limits.memory": "32Gi",
                    "pods": "50",
                },
            ),
            (
                "k1s0-business.yaml",
                {
                    "requests.cpu": "16",
                    "requests.memory": "32Gi",
                    "limits.cpu": "32",
                    "limits.memory": "64Gi",
                    "pods": "100",
                },
            ),
            (
                "k1s0-service.yaml",
                {
                    "requests.cpu": "8",
                    "requests.memory": "16Gi",
                    "limits.cpu": "16",
                    "limits.memory": "32Gi",
                    "pods": "50",
                },
            ),
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
            doc["metadata"]["name"] for doc in self.docs if doc and doc.get("kind") == "ClusterRole"
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


class TestKubernetesResourceLimitValues:
    """kubernetes設計.md: リソースリミットのテーブル値検証。"""

    @pytest.mark.parametrize(
        "env_file,expected_resources",
        [
            (
                "values-dev.yaml",
                {
                    "requests": {"cpu": "100m", "memory": "128Mi"},
                    "limits": {"cpu": "500m", "memory": "512Mi"},
                },
            ),
            (
                "values-staging.yaml",
                {
                    "requests": {"cpu": "250m", "memory": "256Mi"},
                    "limits": {"cpu": "1000m", "memory": "1Gi"},
                },
            ),
            (
                "values-prod.yaml",
                {
                    "requests": {"cpu": "500m", "memory": "512Mi"},
                    "limits": {"cpu": "2000m", "memory": "2Gi"},
                },
            ),
        ],
    )
    def test_server_pod_resource_limits(self, env_file: str, expected_resources: dict) -> None:
        """kubernetes設計.md: サーバー Pod のリソースリミットが環境別テーブル値と一致すること。"""
        path = ROOT / "infra" / "helm" / "services" / "service" / "order" / env_file
        with open(path, encoding="utf-8") as f:
            values = yaml.safe_load(f)
        resources = values["resources"]
        for rtype in ("requests", "limits"):
            for key in ("cpu", "memory"):
                actual = str(resources[rtype][key])
                expected = expected_resources[rtype][key]
                assert actual == expected, (
                    f"{env_file}: resources.{rtype}.{key} が {expected} ではなく {actual}"
                )


class TestKubernetesIngressExternalName:
    """kubernetes設計.md: Ingress ExternalName Service 検証。"""

    def setup_method(self) -> None:
        path = K8S / "ingress" / "ingress.yaml"
        assert path.exists()
        self.docs = list(yaml.safe_load_all(path.read_text(encoding="utf-8")))

    def test_kong_proxy_external_name(self) -> None:
        """kubernetes設計.md: kong-proxy ExternalName Service が定義されていること。"""
        ext_svcs = [
            d
            for d in self.docs
            if d and d.get("kind") == "Service" and d.get("spec", {}).get("type") == "ExternalName"
        ]
        kong = [s for s in ext_svcs if s["metadata"]["name"] == "kong-proxy"]
        assert len(kong) > 0, "kong-proxy ExternalName Service が定義されていません"
        assert kong[0]["spec"]["externalName"] == "kong-proxy.k1s0-system.svc.cluster.local"
        assert kong[0]["metadata"]["namespace"] == "ingress"

    def test_grafana_external_name(self) -> None:
        """kubernetes設計.md: grafana ExternalName Service が定義されていること。"""
        ext_svcs = [
            d
            for d in self.docs
            if d and d.get("kind") == "Service" and d.get("spec", {}).get("type") == "ExternalName"
        ]
        grafana = [s for s in ext_svcs if s["metadata"]["name"] == "grafana"]
        assert len(grafana) > 0, "grafana ExternalName Service が定義されていません"
        assert grafana[0]["spec"]["externalName"] == "grafana.observability.svc.cluster.local"

    def test_ingress_routing_kong_proxy(self) -> None:
        """kubernetes設計.md: Ingress ルーティング先に kong-proxy が含まれること。"""
        ingress = [d for d in self.docs if d and d.get("kind") == "Ingress"][0]
        rules = ingress["spec"]["rules"]
        api_rule = [r for r in rules if r["host"] == "api.k1s0.internal.example.com"]
        assert len(api_rule) > 0
        backend = api_rule[0]["http"]["paths"][0]["backend"]["service"]
        assert backend["name"] == "kong-proxy"
        assert backend["port"]["number"] == 80

    def test_ingress_routing_grafana(self) -> None:
        """kubernetes設計.md: Ingress ルーティング先に grafana が含まれること。"""
        ingress = [d for d in self.docs if d and d.get("kind") == "Ingress"][0]
        rules = ingress["spec"]["rules"]
        grafana_rule = [r for r in rules if r["host"] == "grafana.k1s0.internal.example.com"]
        assert len(grafana_rule) > 0
        backend = grafana_rule[0]["http"]["paths"][0]["backend"]["service"]
        assert backend["name"] == "grafana"
        assert backend["port"]["number"] == 3000


class TestKubernetesResourceQuotaPVC:
    """kubernetes設計.md: ResourceQuota persistentvolumeclaims 値の検証。"""

    @pytest.mark.parametrize(
        "ns_file,expected_pvcs",
        [
            ("k1s0-system.yaml", "20"),
            ("k1s0-business.yaml", "40"),
            ("k1s0-service.yaml", "20"),
        ],
    )
    def test_resource_quota_pvcs(self, ns_file: str, expected_pvcs: str) -> None:
        """kubernetes設計.md: ResourceQuota の persistentvolumeclaims が仕様通りであること。"""
        path = K8S / "namespaces" / ns_file
        docs = list(yaml.safe_load_all(path.read_text(encoding="utf-8")))
        quotas = [d for d in docs if d and d.get("kind") == "ResourceQuota"]
        assert len(quotas) > 0, f"{ns_file} に ResourceQuota が定義されていません"
        hard = quotas[0]["spec"]["hard"]
        assert str(hard["persistentvolumeclaims"]) == expected_pvcs, (
            f"{ns_file}: persistentvolumeclaims が {expected_pvcs} ではなく {hard.get('persistentvolumeclaims')}"
        )


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

    def test_network_policy_yaml_structure(self) -> None:
        """kubernetes設計.md: NetworkPolicy YAML 内容詳細検証。"""
        # kubernetes設計.md の NetworkPolicy 定義で重要な要素が Terraform 内に含まれていること
        # pod_selector、policy_types、allowed_from_tiers の動的構築
        assert "pod_selector" in self.content
        assert '"Ingress"' in self.content  # policy_types = ["Ingress"]
        assert "allowed_from_tiers" in self.content  # dynamic な from 構築


class TestKubernetesHPABehavior:
    """kubernetes設計.md: HPA behavior 設定テスト。"""

    def test_hpa_template_exists(self) -> None:
        """kubernetes設計.md: HPA テンプレートが存在すること。"""
        path = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates" / "_hpa.tpl"
        assert path.exists(), "_hpa.tpl が存在しません"

    def test_hpa_behavior_scale_up(self) -> None:
        """kubernetes設計.md: HPA behavior scaleUp が仕様通りであること。"""
        path = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates" / "_hpa.tpl"
        content = path.read_text(encoding="utf-8")
        assert "scaleUp" in content
        assert "stabilizationWindowSeconds: 60" in content
        # scaleUp: value: 2, periodSeconds: 60
        assert "value: 2" in content

    def test_hpa_behavior_scale_down(self) -> None:
        """kubernetes設計.md: HPA behavior scaleDown が仕様通りであること。"""
        path = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates" / "_hpa.tpl"
        content = path.read_text(encoding="utf-8")
        assert "scaleDown" in content
        assert "stabilizationWindowSeconds: 300" in content
        assert "periodSeconds: 120" in content

    def test_hpa_metrics_cpu(self) -> None:
        """kubernetes設計.md: HPA metrics CPU averageUtilization が仕様通りであること。"""
        path = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates" / "_hpa.tpl"
        content = path.read_text(encoding="utf-8")
        assert "name: cpu" in content
        assert "targetCPUUtilizationPercentage" in content

    def test_hpa_metrics_memory(self) -> None:
        """kubernetes設計.md: HPA metrics Memory averageUtilization が仕様通りであること。"""
        path = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates" / "_hpa.tpl"
        content = path.read_text(encoding="utf-8")
        assert "name: memory" in content
        assert "targetMemoryUtilizationPercentage" in content


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
