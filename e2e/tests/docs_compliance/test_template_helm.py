"""テンプレート仕様-Helm.md の仕様準拠テスト。

Helmテンプレート仕様書とテンプレートファイルの検証。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-Helm.md"
HELM = TEMPLATES / "helm"


class TestHelmSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-Helm.md が存在しません"


class TestHelmTemplateFilesExist:
    """テンプレートファイルの存在確認。"""

    @pytest.mark.parametrize(
        "template",
        [
            "Chart.yaml.tera",
            "values.yaml.tera",
            "values-dev.yaml.tera",
            "values-staging.yaml.tera",
            "values-prod.yaml.tera",
            "templates/deployment.yaml.tera",
            "templates/service.yaml.tera",
            "templates/configmap.yaml.tera",
            "templates/hpa.yaml.tera",
            "templates/pdb.yaml.tera",
        ],
    )
    def test_helm_template_exists(self, template: str) -> None:
        path = HELM / template
        assert path.exists(), f"helm/{template} が存在しません"


class TestHelmSpecSections:
    """仕様書の主要セクション存在チェック。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "section",
        [
            "## 概要",
            "## 配置パス",
            "## テンプレートファイル一覧",
            "## 条件付き生成",
            "## Tera / Helm 構文衝突の回避",
            "## テンプレート変数マッピング",
            "## テンプレート詳細",
            "## テンプレート変数一覧",
        ],
    )
    def test_section_exists(self, section: str) -> None:
        assert section in self.content, f"セクション '{section}' が仕様書に存在しません"


class TestHelmSpecRawSyntax:
    """Tera/Helm構文衝突の回避（{% raw %} の記載）検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_raw_block_documented(self) -> None:
        assert "{% raw %}" in self.content, "{% raw %} が仕様書に記載されていません"

    def test_endraw_block_documented(self) -> None:
        assert "{% endraw %}" in self.content, "{% endraw %} が仕様書に記載されていません"


class TestHelmTemplateVariables:
    """テンプレート変数の使用チェック。"""

    @pytest.mark.parametrize(
        "template,variable",
        [
            ("Chart.yaml.tera", "{{ service_name }}"),
            ("Chart.yaml.tera", "{{ service_name_pascal }}"),
            ("values.yaml.tera", "{{ docker_registry }}"),
            ("values.yaml.tera", "{{ docker_project }}"),
            ("values.yaml.tera", "{{ service_name }}"),
            ("values.yaml.tera", "{{ tier }}"),
            ("values-dev.yaml.tera", "has_database"),
            ("values-staging.yaml.tera", "has_database"),
            ("values-prod.yaml.tera", "{{ service_name }}"),
        ],
    )
    def test_template_variable_used(self, template: str, variable: str) -> None:
        path = HELM / template
        content = path.read_text(encoding="utf-8")
        assert variable in content, f"helm/{template} に変数 '{variable}' が含まれていません"

    @pytest.mark.parametrize(
        "template,include_call",
        [
            ("templates/deployment.yaml.tera", "k1s0-common.deployment"),
            ("templates/service.yaml.tera", "k1s0-common.service"),
            ("templates/configmap.yaml.tera", "k1s0-common.configmap"),
            ("templates/hpa.yaml.tera", "k1s0-common.hpa"),
            ("templates/pdb.yaml.tera", "k1s0-common.pdb"),
        ],
    )
    def test_library_chart_include(self, template: str, include_call: str) -> None:
        path = HELM / template
        content = path.read_text(encoding="utf-8")
        assert include_call in content, (
            f"helm/{template} に Library Chart 呼び出し '{include_call}' が含まれていません"
        )
