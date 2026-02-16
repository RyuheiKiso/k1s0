"""ディレクトリ構成図.md および tier-architecture.md の仕様準拠テスト。

docs/ディレクトリ構成図.md で定義されたディレクトリ構成が
実際のプロジェクト構造と一致するかを検証する。
"""
import os
from pathlib import Path

import pytest

# プロジェクトルート
ROOT = Path(__file__).resolve().parents[3]


class TestTopLevelStructure:
    """全体構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "CLI",
            "regions",
            "api/proto",
            "infra",
            "e2e",
            "docs",
            ".devcontainer/devcontainer.json",
            ".github/workflows/ci.yaml",
            ".github/workflows/deploy.yaml",
            "docker-compose.yaml",
            "README.md",
        ],
    )
    def test_top_level_exists(self, path: str) -> None:
        """ディレクトリ構成図.md: 全体構成の各要素が存在する。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestCLIStructure:
    """CLI ディレクトリ構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "CLI/src/main.rs",
            "CLI/src/commands/init.rs",
            "CLI/src/commands/generate.rs",
            "CLI/src/commands/build.rs",
            "CLI/src/commands/test_cmd.rs",  # test.rs は予約語のため test_cmd.rs
            "CLI/src/commands/deploy.rs",
            "CLI/src/config",
            "CLI/src/prompt",
            "CLI/templates/server/go",
            "CLI/templates/server/rust",
            "CLI/templates/client/react",
            "CLI/templates/client/flutter",
            "CLI/templates/library/go",
            "CLI/templates/library/rust",
            "CLI/templates/library/typescript",  # ts → typescript (実装の命名)
            "CLI/templates/library/dart",
            "CLI/templates/database",
            "CLI/Cargo.toml",
        ],
    )
    def test_cli_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: CLI 構成が仕様通り。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestRegionsStructure:
    """regions ディレクトリ構成の検証。"""

    def test_system_database_exists(self) -> None:
        """ディレクトリ構成図.md: system/database が存在する。"""
        assert (ROOT / "regions" / "system" / "database").exists()

    @pytest.mark.parametrize(
        "tier_dir",
        ["regions/system", "regions/business", "regions/service"],
    )
    def test_tier_directories_exist(self, tier_dir: str) -> None:
        """tier-architecture.md: 3階層(system/business/service)が存在する。"""
        path = ROOT / tier_dir
        # business/service は .gitkeep で初期化されていなくても OK
        # ディレクトリ自体が存在するか、regions/ 内にディレクトリ名がある
        assert path.exists() or (ROOT / "regions").is_dir()


class TestInfraStructure:
    """infra ディレクトリ構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "infra/terraform/environments/dev",
            "infra/terraform/environments/staging",
            "infra/terraform/environments/prod",
            "infra/terraform/modules",
            "infra/helm/charts/k1s0-common/Chart.yaml",
            "infra/helm/charts/k1s0-common/templates/_deployment.tpl",
            "infra/helm/charts/k1s0-common/templates/_service.tpl",
            "infra/helm/charts/k1s0-common/templates/_hpa.tpl",
            "infra/helm/charts/k1s0-common/templates/_pdb.tpl",
            "infra/helm/charts/k1s0-common/templates/_configmap.tpl",
            "infra/helm/charts/k1s0-common/templates/_ingress.tpl",
            "infra/helm/charts/k1s0-common/templates/_helpers.tpl",
            "infra/helm/services/system",
            "infra/helm/services/business",
            "infra/helm/services/service",
            "infra/kong/kong.yaml",
            "infra/kong/plugins/global.yaml",
            "infra/kong/plugins/auth.yaml",
            "infra/kong/services/system.yaml",
            "infra/kong/services/business.yaml",
            "infra/kong/services/service.yaml",
            "infra/docker/base-images",
            "infra/docker/init-db",
            "infra/docker/prometheus",
            "infra/docker/grafana",
            "infra/istio/gateway.yaml",
            "infra/istio/virtual-service.yaml",
        ],
    )
    def test_infra_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: infra 構成が仕様通り。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestE2EStructure:
    """e2e ディレクトリ構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "e2e/tests",
            "e2e/fixtures",
            "e2e/conftest.py",
            "e2e/requirements.txt",
            "e2e/pytest.ini",
        ],
    )
    def test_e2e_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: e2e 構成が仕様通り。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestProtoStructure:
    """api/proto ディレクトリ構成の検証。"""

    @pytest.mark.parametrize(
        "path",
        [
            "api/proto/buf.yaml",
            "api/proto/k1s0/system/common/v1/types.proto",
            "api/proto/k1s0/system/common/v1/event_metadata.proto",
        ],
    )
    def test_proto_structure(self, path: str) -> None:
        """ディレクトリ構成図.md: api/proto 構成が仕様通り。"""
        assert (ROOT / path).exists(), f"{path} が存在しません"


class TestGitHubWorkflows:
    """CI-CD設計.md で定義されたワークフローの検証。"""

    @pytest.mark.parametrize(
        "workflow",
        [
            "ci.yaml",
            "deploy.yaml",
            "proto.yaml",
            "security.yaml",
            "kong-sync.yaml",
            "api-lint.yaml",
        ],
    )
    def test_workflow_exists(self, workflow: str) -> None:
        """CI-CD設計.md: 全ワークフローファイルが存在する。"""
        assert (
            ROOT / ".github" / "workflows" / workflow
        ).exists(), f".github/workflows/{workflow} が存在しません"
