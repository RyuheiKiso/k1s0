"""Library Chart ヘルパーテンプレートの存在検証。"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
CHART_TEMPLATES = ROOT / "infra" / "helm" / "charts" / "k1s0-common" / "templates"


class TestLibraryChartHelperTemplates:
    """k1s0-common Library Chart のヘルパーテンプレート存在検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "_helpers.tpl",
            "_deployment.tpl",
            "_service.tpl",
            "_hpa.tpl",
            "_pdb.tpl",
            "_configmap.tpl",
            "_ingress.tpl",
        ],
    )
    def test_helper_template_exists(self, template: str) -> None:
        path = CHART_TEMPLATES / template
        assert path.exists(), f"k1s0-common/templates/{template} が存在しません"


class TestDeploymentGrpcHealthCheck:
    """_deployment.tpl に gRPC ヘルスチェック条件分岐が存在することを検証。"""

    def test_deployment_has_grpc_health_check(self) -> None:
        """_deployment.tpl に grpcHealthCheck 条件分岐が存在することを検証"""
        path = CHART_TEMPLATES / "_deployment.tpl"
        content = path.read_text(encoding="utf-8")
        assert "grpcHealthCheck" in content, (
            "_deployment.tpl に grpcHealthCheck の条件分岐が含まれていません"
        )

    def test_deployment_grpc_probe_port(self) -> None:
        """_deployment.tpl の gRPC probe に grpc port 参照が含まれることを検証"""
        path = CHART_TEMPLATES / "_deployment.tpl"
        content = path.read_text(encoding="utf-8")
        assert "grpc:" in content or "grpcPort" in content, (
            "_deployment.tpl に gRPC probe のポート設定が含まれていません"
        )
