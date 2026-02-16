"""terraform設計.md の仕様準拠テスト。

infra/terraform/ の構成がドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TF = ROOT / "infra" / "terraform"


class TestTerraformEnvironments:
    """terraform設計.md: 環境別ディレクトリの検証。"""

    @pytest.mark.parametrize("env", ["dev", "staging", "prod"])
    def test_environment_exists(self, env: str) -> None:
        assert (TF / "environments" / env).exists()

    @pytest.mark.parametrize(
        "env,file",
        [
            ("dev", "main.tf"),
            ("dev", "variables.tf"),
            ("dev", "terraform.tfvars"),
            ("dev", "backend.tf"),
            ("dev", "outputs.tf"),
            ("staging", "main.tf"),
            ("staging", "variables.tf"),
            ("staging", "terraform.tfvars"),
            ("staging", "backend.tf"),
            ("staging", "outputs.tf"),
            ("prod", "main.tf"),
            ("prod", "variables.tf"),
            ("prod", "terraform.tfvars"),
            ("prod", "backend.tf"),
            ("prod", "outputs.tf"),
        ],
    )
    def test_environment_files(self, env: str, file: str) -> None:
        path = TF / "environments" / env / file
        assert path.exists(), f"environments/{env}/{file} が存在しません"


class TestTerraformModules:
    """terraform設計.md: モジュールの検証。"""

    @pytest.mark.parametrize(
        "module",
        [
            "kubernetes-base",
            "kubernetes-storage",
            "observability",
            "messaging",
            "database",
            "vault",
            "harbor",
            "service-mesh",
        ],
    )
    def test_module_exists(self, module: str) -> None:
        path = TF / "modules" / module
        assert path.exists(), f"modules/{module}/ が存在しません"

    @pytest.mark.parametrize(
        "module",
        [
            "kubernetes-base",
            "kubernetes-storage",
            "observability",
            "messaging",
            "database",
            "vault",
            "harbor",
            "service-mesh",
        ],
    )
    def test_module_has_main_tf(self, module: str) -> None:
        """各モジュールに main.tf が存在すること。"""
        path = TF / "modules" / module / "main.tf"
        assert path.exists(), f"modules/{module}/main.tf が存在しません"


class TestTerraformBackend:
    """terraform設計.md: State管理の検証。"""

    @pytest.mark.parametrize("env", ["dev", "staging", "prod"])
    def test_backend_uses_consul(self, env: str) -> None:
        backend = TF / "environments" / env / "backend.tf"
        content = backend.read_text(encoding="utf-8")
        assert 'backend "consul"' in content, f"{env}/backend.tf が Consul バックエンドを使用していません"

    def test_dev_backend_path(self) -> None:
        content = (TF / "environments" / "dev" / "backend.tf").read_text(encoding="utf-8")
        assert "terraform/k1s0/dev" in content

    def test_staging_backend_path(self) -> None:
        content = (TF / "environments" / "staging" / "backend.tf").read_text(encoding="utf-8")
        assert "terraform/k1s0/staging" in content

    def test_prod_backend_path(self) -> None:
        content = (TF / "environments" / "prod" / "backend.tf").read_text(encoding="utf-8")
        assert "terraform/k1s0/prod" in content


class TestTerraformKubernetesBase:
    """terraform設計.md: kubernetes-base モジュールの内容検証。"""

    def setup_method(self) -> None:
        path = TF / "modules" / "kubernetes-base" / "main.tf"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_namespace_resource(self) -> None:
        assert "kubernetes_namespace" in self.content

    def test_network_policy_resource(self) -> None:
        assert "kubernetes_network_policy" in self.content


class TestTerraformKubernetesStorage:
    """terraform設計.md: kubernetes-storage モジュールの内容検証。"""

    def setup_method(self) -> None:
        path = TF / "modules" / "kubernetes-storage" / "main.tf"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_ceph_block_storage_class(self) -> None:
        assert "ceph-block" in self.content or "ceph_block" in self.content

    def test_ceph_filesystem_storage_class(self) -> None:
        assert "ceph-filesystem" in self.content or "ceph_filesystem" in self.content

    def test_ceph_block_fast_storage_class(self) -> None:
        assert "ceph-block-fast" in self.content or "ceph_block_fast" in self.content

    def test_rbd_csi_provisioner(self) -> None:
        assert "rbd.csi.ceph.com" in self.content


class TestTerraformServiceMesh:
    """terraform設計.md: service-mesh モジュールの内容検証。"""

    def setup_method(self) -> None:
        self.module_dir = TF / "modules" / "service-mesh"
        assert self.module_dir.exists()

    def test_main_tf_has_istio(self) -> None:
        content = (self.module_dir / "main.tf").read_text(encoding="utf-8")
        assert "istio" in content.lower()

    def test_kiali_tf_exists(self) -> None:
        assert (self.module_dir / "kiali.tf").exists()

    def test_flagger_tf_exists(self) -> None:
        assert (self.module_dir / "flagger.tf").exists()

    def test_flagger_mesh_provider(self) -> None:
        content = (self.module_dir / "flagger.tf").read_text(encoding="utf-8")
        assert "istio" in content


class TestTerraformVault:
    """terraform設計.md: vault モジュールの内容検証。"""

    def setup_method(self) -> None:
        self.module_dir = TF / "modules" / "vault"
        assert self.module_dir.exists()

    def test_auth_tf_exists(self) -> None:
        assert (self.module_dir / "auth.tf").exists()

    def test_secrets_tf_exists(self) -> None:
        assert (self.module_dir / "secrets.tf").exists()

    def test_policies_tf_exists(self) -> None:
        assert (self.module_dir / "policies.tf").exists()

    @pytest.mark.parametrize("policy", ["system.hcl", "business.hcl", "service.hcl"])
    def test_policy_files_exist(self, policy: str) -> None:
        path = self.module_dir / "policies" / policy
        assert path.exists(), f"policies/{policy} が存在しません"


class TestTerraformDevTfvars:
    """terraform設計.md: dev/terraform.tfvars の内容検証。"""

    def setup_method(self) -> None:
        path = TF / "environments" / "dev" / "terraform.tfvars"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "namespace",
        [
            "k1s0-system",
            "k1s0-business",
            "k1s0-service",
            "observability",
            "messaging",
            "ingress",
            "service-mesh",
            "cert-manager",
            "harbor",
        ],
    )
    def test_namespace_defined(self, namespace: str) -> None:
        assert namespace in self.content, f"Namespace '{namespace}' が定義されていません"

    def test_reclaim_policy_delete(self) -> None:
        assert "Delete" in self.content


class TestTerraformProdTfvars:
    """terraform設計.md: prod/terraform.tfvars の内容検証。"""

    def test_reclaim_policy_retain(self) -> None:
        content = (TF / "environments" / "prod" / "terraform.tfvars").read_text(encoding="utf-8")
        assert "Retain" in content


class TestTerraformObservabilityHelmReleases:
    """terraform設計.md: observability モジュールの helm_release 定義テスト。"""

    def setup_method(self) -> None:
        path = TF / "modules" / "observability" / "main.tf"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "release_name",
        ["prometheus", "loki", "jaeger"],
    )
    def test_helm_release_defined(self, release_name: str) -> None:
        """terraform設計.md: observability モジュールに helm_release が定義されていること。"""
        assert f'helm_release" "{release_name}"' in self.content, (
            f"helm_release '{release_name}' が observability/main.tf に定義されていません"
        )

    def test_prometheus_chart(self) -> None:
        assert "kube-prometheus-stack" in self.content

    def test_loki_chart(self) -> None:
        assert "loki-stack" in self.content

    def test_jaeger_chart(self) -> None:
        assert '"jaeger"' in self.content


class TestTerraformDatabaseBackup:
    """terraform設計.md: database モジュールの backup.tf 存在テスト。"""

    def test_backup_tf_exists(self) -> None:
        path = TF / "modules" / "database" / "backup.tf"
        assert path.exists(), "modules/database/backup.tf が存在しません"

    def test_backup_tf_has_cronjob(self) -> None:
        content = (TF / "modules" / "database" / "backup.tf").read_text(encoding="utf-8")
        assert "kubernetes_cron_job_v1" in content


class TestTerraformHarborProjects:
    """terraform設計.md: harbor モジュールの projects.tf テスト。"""

    def test_projects_tf_exists(self) -> None:
        path = TF / "modules" / "harbor" / "projects.tf"
        assert path.exists(), "modules/harbor/projects.tf が存在しません"

    @pytest.mark.parametrize(
        "project",
        ["k1s0-system", "k1s0-business", "k1s0-service", "k1s0-infra"],
    )
    def test_harbor_project_defined(self, project: str) -> None:
        """terraform設計.md: 4 プロジェクトが定義されていること。"""
        content = (TF / "modules" / "harbor" / "projects.tf").read_text(encoding="utf-8")
        assert project in content, f"Harbor プロジェクト '{project}' が定義されていません"
