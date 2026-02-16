"""テンプレート仕様-ライブラリ.md の内容準拠テスト。

CLI/templates/library/ のテンプレートファイルの内容が
仕様ドキュメントのコードブロックと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"
LIB_GO = TEMPLATES / "library" / "go"
LIB_RUST = TEMPLATES / "library" / "rust"
LIB_TS = TEMPLATES / "library" / "typescript"
LIB_DART = TEMPLATES / "library" / "dart"


class TestGoLibModContent:
    """テンプレート仕様-ライブラリ.md: Go go.mod.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_GO / "go.mod.tera").read_text(encoding="utf-8")

    def test_module_variable(self) -> None:
        assert "{{ go_module }}" in self.content

    def test_go_version(self) -> None:
        assert "go 1.23" in self.content

    def test_minimal_dependencies(self) -> None:
        """テンプレート仕様-ライブラリ.md: 必要最小限の依存。"""
        lines = self.content.strip().split("\n")
        assert len(lines) <= 5, "ライブラリ go.mod は最小限の内容であるべき"


class TestGoLibPublicPackageContent:
    """テンプレート仕様-ライブラリ.md: {name}.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_GO / "{name}.go.tera").read_text(encoding="utf-8")

    def test_package_declaration(self) -> None:
        assert "package {{ service_name | snake_case }}" in self.content

    def test_has_todo(self) -> None:
        assert "TODO" in self.content


class TestGoLibInternalContent:
    """テンプレート仕様-ライブラリ.md: internal/internal.go.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_GO / "internal" / "internal.go.tera").read_text(encoding="utf-8")

    def test_package_internal(self) -> None:
        assert "package internal" in self.content


class TestGoLibTestContent:
    """テンプレート仕様-ライブラリ.md: テストファイルの検証。"""

    def test_unit_test_exists(self) -> None:
        assert (LIB_GO / "{name}_test.go.tera").exists()

    def test_integration_test_exists(self) -> None:
        assert (LIB_GO / "tests" / "integration_test.go.tera").exists()

    def test_readme_exists(self) -> None:
        assert (LIB_GO / "README.md.tera").exists()


class TestRustLibCargoTomlContent:
    """テンプレート仕様-ライブラリ.md: Rust Cargo.toml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_RUST / "Cargo.toml.tera").read_text(encoding="utf-8")

    def test_package_name(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_edition(self) -> None:
        assert 'edition = "2021"' in self.content

    def test_serde(self) -> None:
        assert "serde" in self.content

    def test_thiserror(self) -> None:
        assert "thiserror" in self.content

    def test_mockall_dev(self) -> None:
        assert "[dev-dependencies]" in self.content
        assert "mockall" in self.content


class TestRustLibSrcContent:
    """テンプレート仕様-ライブラリ.md: Rust src テンプレートの内容検証。"""

    def test_lib_rs(self) -> None:
        content = (LIB_RUST / "src" / "lib.rs.tera").read_text(encoding="utf-8")
        assert "pub mod" in content

    def test_module_rs(self) -> None:
        content = (LIB_RUST / "src" / "{module}.rs.tera").read_text(encoding="utf-8")
        assert "#[cfg(test)]" in content
        assert "mod tests" in content


class TestRustLibTestContent:
    """テンプレート仕様-ライブラリ.md: Rust テストファイルの検証。"""

    def test_integration_test_exists(self) -> None:
        assert (LIB_RUST / "tests" / "integration_test.rs.tera").exists()

    def test_readme_exists(self) -> None:
        assert (LIB_RUST / "README.md.tera").exists()


class TestTsLibPackageJsonContent:
    """テンプレート仕様-ライブラリ.md: TypeScript package.json.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_TS / "package.json.tera").read_text(encoding="utf-8")

    def test_service_name(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_main_entry(self) -> None:
        assert '"main"' in self.content
        assert "dist/index.js" in self.content

    def test_types_entry(self) -> None:
        assert '"types"' in self.content
        assert "dist/index.d.ts" in self.content

    def test_scripts(self) -> None:
        assert '"build"' in self.content
        assert '"test"' in self.content

    def test_dev_dependencies(self) -> None:
        assert "typescript" in self.content
        assert "vitest" in self.content
        assert "eslint" in self.content
        assert "prettier" in self.content


class TestTsLibTsconfigContent:
    """テンプレート仕様-ライブラリ.md: TypeScript tsconfig.json.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_TS / "tsconfig.json.tera").read_text(encoding="utf-8")

    def test_strict(self) -> None:
        assert '"strict": true' in self.content

    def test_declaration(self) -> None:
        assert '"declaration": true' in self.content

    def test_out_dir(self) -> None:
        assert '"outDir": "dist"' in self.content


class TestTsLibSrcContent:
    """テンプレート仕様-ライブラリ.md: TypeScript src/index.ts.tera の内容検証。"""

    def test_index_ts_exists(self) -> None:
        assert (LIB_TS / "src" / "index.ts.tera").exists()

    def test_index_ts_has_export_comment(self) -> None:
        content = (LIB_TS / "src" / "index.ts.tera").read_text(encoding="utf-8")
        assert "{{ service_name }}" in content


class TestTsLibTestContent:
    """テンプレート仕様-ライブラリ.md: TypeScript テストファイルの検証。"""

    def test_test_file_exists(self) -> None:
        assert (LIB_TS / "tests" / "index.test.ts.tera").exists()

    def test_readme_exists(self) -> None:
        assert (LIB_TS / "README.md.tera").exists()


class TestDartLibPubspecContent:
    """テンプレート仕様-ライブラリ.md: Dart pubspec.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_DART / "pubspec.yaml.tera").read_text(encoding="utf-8")

    def test_service_name_snake(self) -> None:
        assert "{{ service_name | snake_case }}" in self.content

    def test_sdk_constraint(self) -> None:
        assert ">=3.0.0 <4.0.0" in self.content

    def test_mocktail_dev(self) -> None:
        assert "mocktail" in self.content


class TestDartLibSrcContent:
    """テンプレート仕様-ライブラリ.md: Dart lib テンプレートの内容検証。"""

    def test_entry_point(self) -> None:
        content = (LIB_DART / "lib" / "{name}.dart.tera").read_text(encoding="utf-8")
        assert "library" in content

    def test_src_module(self) -> None:
        content = (LIB_DART / "lib" / "src" / "{module}.dart.tera").read_text(encoding="utf-8")
        assert "{{ service_name | snake_case }}" in content


class TestDartLibTestAndAnalysis:
    """テンプレート仕様-ライブラリ.md: Dart テスト・lint ファイルの検証。"""

    def test_analysis_options_exists(self) -> None:
        assert (LIB_DART / "analysis_options.yaml.tera").exists()

    def test_test_file_exists(self) -> None:
        """テンプレート仕様-ライブラリ.md: test/ ディレクトリ (tests/ ではない)。"""
        assert (LIB_DART / "test" / "{module}_test.dart.tera").exists()

    def test_readme_exists(self) -> None:
        assert (LIB_DART / "README.md.tera").exists()
