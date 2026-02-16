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
