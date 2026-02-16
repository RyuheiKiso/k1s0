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
