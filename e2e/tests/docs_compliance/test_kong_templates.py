"""Kong API Gateway テンプレートの仕様準拠テスト。

テンプレートファイルの存在と仕様書との一致を検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-Kong.md"


class TestKongSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-Kong.md が存在しません"


class TestKongTemplateFiles:
    """Kong テンプレートファイルの存在確認。"""

    @pytest.mark.parametrize(
        "template",
        [
            "kong-service.yaml.tera",
            "kong-plugins.yaml.tera",
        ],
    )
    def test_kong_template_exists(self, template: str) -> None:
        path = TEMPLATES / "kong" / template
        assert path.exists(), f"kong/{template} が存在しません"


class TestKongSpecSections:
    """仕様書に主要セクションが存在するかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "section",
        [
            "## 概要",
            "## テンプレートファイル一覧",
            "## 使用するテンプレート変数",
            "## 関連ドキュメント",
        ],
    )
    def test_section_exists(self, section: str) -> None:
        assert section in self.content, f"セクション '{section}' が仕様書に存在しません"


class TestKongServiceTemplate:
    """kong-service.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "kong" / "kong-service.yaml.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_has_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_has_namespace_variable(self) -> None:
        assert "{{ namespace }}" in self.content

    def test_has_server_port_variable(self) -> None:
        assert "{{ server_port }}" in self.content

    def test_has_ingress_class(self) -> None:
        assert "kong" in self.content


class TestKongPluginsTemplate:
    """kong-plugins.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "kong" / "kong-plugins.yaml.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_has_rate_limiting(self) -> None:
        assert "rate-limiting" in self.content

    def test_has_cors(self) -> None:
        assert "cors" in self.content

    def test_has_jwt(self) -> None:
        assert "jwt" in self.content

    def test_has_tier_conditional(self) -> None:
        assert "tier" in self.content
