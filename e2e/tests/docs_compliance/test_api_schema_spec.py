"""テンプレート仕様-APIスキーマ.md の仕様準拠テスト。

API スキーマテンプレート（OpenAPI、Proto、GraphQL）の
仕様書とテンプレートファイルの一致を検証する。
"""

from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-APIスキーマ.md"


class TestAPISchemaSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-APIスキーマ.md が存在しません"


class TestAPISchemaSpecSections:
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


class TestOpenAPITemplate:
    """OpenAPI テンプレートの検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "server" / "go" / "api" / "openapi" / "openapi.yaml.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_has_openapi_version(self) -> None:
        assert "openapi" in self.content.lower() or "3.0" in self.content

    def test_has_service_name_variable(self) -> None:
        assert "service_name" in self.content

    def test_has_paths(self) -> None:
        assert "paths" in self.content


class TestProtoTemplate:
    """Protocol Buffers テンプレートの検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "server" / "go" / "api" / "proto" / "service.proto.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_has_syntax_proto3(self) -> None:
        assert "proto3" in self.content

    def test_has_service_definition(self) -> None:
        assert "service" in self.content

    def test_has_package(self) -> None:
        assert "package" in self.content


class TestGraphQLTemplate:
    """GraphQL テンプレートの検証。"""

    def setup_method(self) -> None:
        path = TEMPLATES / "server" / "go" / "api" / "graphql" / "schema.graphql.tera"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")

    def test_has_query_type(self) -> None:
        assert "type Query" in self.content or "Query" in self.content

    def test_has_input_type(self) -> None:
        assert "input" in self.content or "Input" in self.content

    def test_has_service_name(self) -> None:
        assert "service_name" in self.content or "service_name_pascal" in self.content
