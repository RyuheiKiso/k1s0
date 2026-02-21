"""テンプレート仕様-レンダリングテスト.md の仕様準拠テスト。

レンダリングテスト仕様書の構造・内容が正しいかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-レンダリングテスト.md"
RUST_SPEC = DOCS / "テンプレート仕様-レンダリングテスト-Rust.md"
E2E_SPEC = DOCS / "テンプレート仕様-レンダリングテスト-E2E.md"


class TestRenderingSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-レンダリングテスト.md が存在しません"


class TestRenderingSpecSections:
    """仕様書に主要セクションが存在するかの検証。"""

    def setup_method(self) -> None:
        # 分割されたドキュメントを結合して検証する
        self.content = (
            SPEC.read_text(encoding="utf-8") + "\n"
            + RUST_SPEC.read_text(encoding="utf-8") + "\n"
            + E2E_SPEC.read_text(encoding="utf-8")
        )

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
        # テスト命名規則はメインの仕様書に記載
        self.content = SPEC.read_text(encoding="utf-8")

    def test_naming_pattern_documented(self) -> None:
        assert "test_{kind}_{language}_{feature}" in self.content, (
            "テスト命名規則 'test_{kind}_{language}_{feature}' が記載されていません"
        )


class TestRenderingSpecHelpers:
    """ヘルパー関数パターンが記載されているかの検証。"""

    def setup_method(self) -> None:
        # ヘルパー関数パターンは Rust 統合テスト仕様に記載
        self.content = RUST_SPEC.read_text(encoding="utf-8")

    def test_template_context_builder(self) -> None:
        assert "TemplateContextBuilder" in self.content

    def test_template_engine(self) -> None:
        assert "TemplateEngine" in self.content

    def test_temp_dir(self) -> None:
        assert "TempDir" in self.content


# --- ギャップ 1: テスト2層構成(Rust統合テスト + E2Eテスト) ---


class TestRenderingSpecTwoLayerStructure:
    """テスト2層構成が仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_rust_integration_layer(self) -> None:
        """テンプレート仕様-レンダリングテスト.md: Rust 統合テスト層が記載されている。"""
        assert "Rust 統合テスト" in self.content
        assert "cargo test" in self.content

    def test_e2e_layer(self) -> None:
        """テンプレート仕様-レンダリングテスト.md: E2E テスト層が記載されている。"""
        assert "E2E テスト" in self.content
        assert "Python + pytest" in self.content

    def test_two_layer_table(self) -> None:
        """テンプレート仕様-レンダリングテスト.md: 2層構成テーブルが存在する。"""
        assert "CLI/tests/" in self.content
        assert "e2e/tests/docs_compliance/" in self.content


# --- ギャップ 2: Rust統合テストの5パターン検証 ---


class TestRenderingSpecRustTestPatterns:
    """Rust統合テストの5つのパターンが仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        # Rust テストパターンは Rust 統合テスト仕様に記載
        self.content = RUST_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "pattern_num,keyword",
        [
            ("1", "ファイルリスト検証"),
            ("2", "条件付きファイル検証"),
            ("3", "テンプレート変数置換検証"),
            ("4", "Tera 構文残留チェック"),
            ("5", "設定ファイル内容検証"),
        ],
    )
    def test_rust_test_pattern(self, pattern_num: str, keyword: str) -> None:
        """テンプレート仕様-レンダリングテスト.md: 5つのテストパターンが記載されている。"""
        assert keyword in self.content, (
            f"テストパターン {pattern_num}: '{keyword}' が仕様書に記載されていません"
        )


# --- ギャップ 3: テストデータ定義 ---


class TestRenderingSpecTestData:
    """テストデータ定義が仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_standard_context_section(self) -> None:
        """テンプレート仕様-レンダリングテスト.md: 標準コンテキストセクションが存在する。"""
        assert "### 標準コンテキスト" in self.content

    @pytest.mark.parametrize(
        "variable,default_value",
        [
            ("service_name", "order-api"),
            ("service_name_snake", "order_api"),
            ("service_name_pascal", "OrderApi"),
            ("service_name_camel", "orderApi"),
            ("tier", "service"),
            ("docker_registry", "harbor.internal.example.com"),
            ("docker_project", "k1s0-service"),
        ],
    )
    def test_default_value_documented(self, variable: str, default_value: str) -> None:
        """テンプレート仕様-レンダリングテスト.md: テスト用デフォルト値が記載されている。"""
        assert default_value in self.content

    def test_variable_substitution_context(self) -> None:
        """テンプレート仕様-レンダリングテスト.md: 変数置換テスト用コンテキスト。"""
        assert "### 変数置換テスト用コンテキスト" in self.content
        assert "user-auth" in self.content
        assert "user_auth" in self.content
        assert "UserAuth" in self.content
        assert "userAuth" in self.content


# --- ギャップ 4: E2Eテスト構造パターン ---


class TestRenderingSpecE2EPattern:
    """E2Eテスト構造パターンが仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        # E2E テスト構造パターンとテストクラス名は E2E テスト仕様に記載
        self.content = E2E_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "element",
        [
            "ROOT",
            "TEMPLATES",
            "setup_method",
            "pytest.mark.parametrize",
        ],
    )
    def test_pattern_element(self, element: str) -> None:
        """テンプレート仕様-レンダリングテスト.md: E2Eテストパターン要素が記載されている。"""
        assert element in self.content

    @pytest.mark.parametrize(
        "test_class",
        [
            "TestServerGoTemplates",
            "TestServerRustTemplates",
            "TestClientReactTemplates",
            "TestClientFlutterTemplates",
            "TestLibraryGoTemplates",
            "TestLibraryRustTemplates",
            "TestLibraryTypescriptTemplates",
            "TestLibraryDartTemplates",
            "TestDatabasePostgresqlTemplates",
            "TestDatabaseMysqlTemplates",
            "TestDatabaseSqliteTemplates",
        ],
    )
    def test_e2e_test_class_documented(self, test_class: str) -> None:
        """テンプレート仕様-レンダリングテスト.md: E2Eテストクラスが記載されている。"""
        assert test_class in self.content, (
            f"テストクラス '{test_class}' が仕様書に記載されていません"
        )


# --- ギャップ 5: テスト対象マトリクスの各オプション検証 ---


class TestRenderingSpecMatrixOptions:
    """テスト対象マトリクスの各オプションが記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "kind,language",
        [
            ("server", "go"),
            ("server", "rust"),
            ("client", "react"),
            ("client", "flutter"),
            ("library", "go"),
            ("library", "rust"),
            ("library", "ts"),
            ("library", "dart"),
            ("database", "postgresql"),
            ("database", "mysql"),
            ("database", "sqlite"),
        ],
    )
    def test_matrix_entry(self, kind: str, language: str) -> None:
        """テンプレート仕様-レンダリングテスト.md: マトリクスの各エントリが存在する。"""
        assert kind in self.content
        assert language in self.content

    @pytest.mark.parametrize(
        "option",
        [
            "REST",
            "gRPC",
            "GraphQL",
            "DB",
            "Kafka",
            "Redis",
        ],
    )
    def test_server_options(self, option: str) -> None:
        """テンプレート仕様-レンダリングテスト.md: サーバーオプションがマトリクスに記載。"""
        assert option in self.content


# --- ギャップ 6: 新規テスト例の存在検証 ---


class TestRenderingSpecNewTestExamples:
    """新規テスト例(追加予定)が仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        # 新規テスト例のメソッド名は E2E テスト仕様に記載
        self.content = E2E_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "test_name",
        [
            "test_client_react_file_list",
            "test_client_react_package_json",
            "test_client_flutter_file_list",
            "test_client_flutter_pubspec",
            "test_library_go_file_list",
            "test_library_go_module_path",
            "test_library_rust_file_list",
            "test_library_typescript_file_list",
            "test_library_dart_file_list",
            "test_database_postgresql_file_list",
            "test_database_mysql_file_list",
            "test_database_sqlite_file_list",
        ],
    )
    def test_new_test_example_documented(self, test_name: str) -> None:
        """テンプレート仕様-レンダリングテスト.md: 新規テスト例が記載されている。"""
        assert test_name in self.content, (
            f"新規テスト例 '{test_name}' が仕様書に記載されていません"
        )
