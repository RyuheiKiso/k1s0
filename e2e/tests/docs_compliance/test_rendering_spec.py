"""テンプレート仕様-レンダリングテスト.md の仕様準拠テスト。

レンダリングテスト仕様書の構造・内容が正しいかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-レンダリングテスト.md"


class TestRenderingSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-レンダリングテスト.md が存在しません"


class TestRenderingSpecSections:
    """仕様書に主要セクションが存在するかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "section",
        [
            "## 概要",
            "## テスト対象マトリクス",
            "## Rust 統合テスト仕様",
            "## E2E テスト仕様",
            "## テストデータ定義",
            "## テスト命名規則",
        ],
    )
    def test_section_exists(self, section: str) -> None:
        assert section in self.content, f"セクション '{section}' が仕様書に存在しません"


class TestRenderingSpecMatrix:
    """テスト対象マトリクスが全kindを含むかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "kind",
        ["server", "client", "library", "database"],
    )
    def test_kind_in_matrix(self, kind: str) -> None:
        assert kind in self.content, f"kind '{kind}' がテスト対象マトリクスに存在しません"


class TestRenderingSpecNamingConvention:
    """テスト命名規則の検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_naming_pattern_documented(self) -> None:
        assert "test_{kind}_{language}_{feature}" in self.content, (
            "テスト命名規則 'test_{kind}_{language}_{feature}' が記載されていません"
        )


class TestRenderingSpecHelpers:
    """ヘルパー関数パターンが記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_template_context_builder(self) -> None:
        assert "TemplateContextBuilder" in self.content

    def test_template_engine(self) -> None:
        assert "TemplateEngine" in self.content

    def test_temp_dir(self) -> None:
        assert "TempDir" in self.content
