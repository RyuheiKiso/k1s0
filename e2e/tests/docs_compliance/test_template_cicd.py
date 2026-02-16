"""テンプレート仕様-CICD.md の仕様準拠テスト。

CICDテンプレート仕様書とテンプレートファイルの検証。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-CICD.md"
CICD = TEMPLATES / "cicd"


class TestCicdSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-CICD.md が存在しません"


class TestCicdTemplateFilesExist:
    """テンプレートファイルの存在確認。"""

    @pytest.mark.parametrize(
        "template",
        [
            "ci.yaml.tera",
            "deploy.yaml.tera",
        ],
    )
    def test_cicd_template_exists(self, template: str) -> None:
        path = CICD / template
        assert path.exists(), f"cicd/{template} が存在しません"


class TestCicdSpecSections:
    """仕様書の主要セクション存在チェック。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "section",
        [
            "## 概要",
            "## 生成対象",
            "## 配置パス",
            "## テンプレートファイル一覧",
            "## 使用するテンプレート変数",
            "## GitHub Actions / Tera 構文衝突の回避",
            "## CI ワークフロー仕様",
            "## Deploy ワークフロー仕様",
            "## 言語バージョン",
        ],
    )
    def test_section_exists(self, section: str) -> None:
        assert section in self.content, f"セクション '{section}' が仕様書に存在しません"


class TestCicdSpecRawSyntax:
    """GitHub Actions/Tera構文衝突の回避（{% raw %} の記載）検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_raw_block_documented(self) -> None:
        assert "{% raw %}" in self.content, "{% raw %} が仕様書に記載されていません"

    def test_endraw_block_documented(self) -> None:
        assert "{% endraw %}" in self.content, "{% endraw %} が仕様書に記載されていません"


class TestCicdTemplateVariables:
    """テンプレート変数の使用チェック。"""

    @pytest.mark.parametrize(
        "template,variable",
        [
            ("ci.yaml.tera", "{{ service_name }}"),
            ("ci.yaml.tera", "{{ module_path }}"),
            ("deploy.yaml.tera", "{{ service_name }}"),
            ("deploy.yaml.tera", "{{ module_path }}"),
            ("deploy.yaml.tera", "{{ docker_project }}"),
        ],
    )
    def test_template_variable_used(self, template: str, variable: str) -> None:
        path = CICD / template
        content = path.read_text(encoding="utf-8")
        assert variable in content, f"cicd/{template} に変数 '{variable}' が含まれていません"


class TestCicdSpecGenerationTarget:
    """生成対象の検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "kind",
        ["server", "client", "library", "database"],
    )
    def test_kind_documented(self, kind: str) -> None:
        assert kind in self.content, f"kind '{kind}' が生成対象に記載されていません"

    def test_deploy_server_only(self) -> None:
        assert "server" in self.content
        assert "Deploy" in self.content


class TestCicdSpecLanguageVersions:
    """言語バージョンの記載チェック。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "tool,version",
        [
            ("Go", "1.23"),
            ("Rust", "1.82"),
            ("Node.js", "22"),
            ("Flutter", "3.24.0"),
            ("Helm", "3.16"),
        ],
    )
    def test_version_documented(self, tool: str, version: str) -> None:
        assert version in self.content, f"{tool} のバージョン '{version}' が仕様書に記載されていません"
