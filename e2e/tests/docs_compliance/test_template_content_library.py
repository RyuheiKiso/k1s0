"""テンプレート仕様-ライブラリ.md の内容準拠テスト。

CLI/templates/library/ のテンプレートファイルの内容が
仕様ドキュメントのコードブロックと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"
LIB_GO = TEMPLATES / "library" / "go"
LIB_RUST = TEMPLATES / "library" / "rust"
LIB_TS = TEMPLATES / "library" / "typescript"
LIB_DART = TEMPLATES / "library" / "dart"
LIB_SWIFT = TEMPLATES / "library" / "swift"


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

    def test_has_public_api(self) -> None:
        assert "Config" in self.content
        assert "Client" in self.content
        assert "func New" in self.content


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


# ============================================================================
# テンプレート仕様-ライブラリ.md ギャップ補完テスト (9件)
# ============================================================================


class TestLibraryTierPlacement:
    """テンプレート仕様-ライブラリ.md: 配置 Tier ルール（service 層にライブラリ不可）。"""

    def test_service_tier_no_library(self) -> None:
        """テンプレート仕様-ライブラリ.md: service 層にはライブラリを置かない。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "service 層にはライブラリを置かない" in docs_content

    def test_system_and_business_only(self) -> None:
        """テンプレート仕様-ライブラリ.md: system と business の2階層にのみ配置。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "**system** と **business** の2階層にのみ配置" in docs_content


class TestGoLibTestTooling:
    """テンプレート仕様-ライブラリ.md: Go テストツール（testify/gomock）の使用検証。"""

    def test_testify_in_integration_test(self) -> None:
        """テンプレート仕様-ライブラリ.md: 統合テストに testify を使用。"""
        content = (LIB_GO / "tests" / "integration_test.go.tera").read_text(encoding="utf-8")
        assert "testify" in content

    def test_testify_assert(self) -> None:
        """テンプレート仕様-ライブラリ.md: testify/assert を使用。"""
        content = (LIB_GO / "tests" / "integration_test.go.tera").read_text(encoding="utf-8")
        assert "testify/assert" in content

    def test_docs_mention_gomock(self) -> None:
        """テンプレート仕様-ライブラリ.md: ドキュメントに gomock の記載がある。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "gomock" in docs_content


class TestRustLibLintTools:
    """テンプレート仕様-ライブラリ.md: Rust の rustfmt + clippy 検証。"""

    def test_docs_mention_rustfmt(self) -> None:
        """テンプレート仕様-ライブラリ.md: rustfmt が記載されている。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "rustfmt" in docs_content

    def test_docs_mention_clippy(self) -> None:
        """テンプレート仕様-ライブラリ.md: clippy が記載されている。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "clippy" in docs_content


class TestTsLibLintTools:
    """テンプレート仕様-ライブラリ.md: TypeScript の ESLint + Prettier 検証。"""

    def test_eslint_in_package_json(self) -> None:
        """テンプレート仕様-ライブラリ.md: package.json に eslint が含まれる。"""
        content = (LIB_TS / "package.json.tera").read_text(encoding="utf-8")
        assert "eslint" in content

    def test_prettier_in_package_json(self) -> None:
        """テンプレート仕様-ライブラリ.md: package.json に prettier が含まれる。"""
        content = (LIB_TS / "package.json.tera").read_text(encoding="utf-8")
        assert "prettier" in content

    def test_lint_script(self) -> None:
        """テンプレート仕様-ライブラリ.md: lint スクリプトが定義されている。"""
        content = (LIB_TS / "package.json.tera").read_text(encoding="utf-8")
        assert '"lint"' in content

    def test_format_script(self) -> None:
        """テンプレート仕様-ライブラリ.md: format スクリプトが定義されている。"""
        content = (LIB_TS / "package.json.tera").read_text(encoding="utf-8")
        assert '"format"' in content


class TestDartLibLintTools:
    """テンプレート仕様-ライブラリ.md: Dart の dart analyze + dart format 検証。"""

    def test_docs_mention_dart_analyze(self) -> None:
        """テンプレート仕様-ライブラリ.md: dart analyze が記載されている。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "dart analyze" in docs_content

    def test_docs_mention_dart_format(self) -> None:
        """テンプレート仕様-ライブラリ.md: dart format が記載されている。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "dart format" in docs_content


class TestLibraryCommonGuidelines:
    """テンプレート仕様-ライブラリ.md: 共通ガイドライン検証。"""

    def test_dependency_minimization(self) -> None:
        """テンプレート仕様-ライブラリ.md: 依存の最小化ガイドライン。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "依存の最小化" in docs_content

    def test_public_api_explicit(self) -> None:
        """テンプレート仕様-ライブラリ.md: 公開 API の明示ガイドライン。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "公開 API の明示" in docs_content

    def test_implementation_hiding(self) -> None:
        """テンプレート仕様-ライブラリ.md: 実装の隠蔽ガイドライン。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "実装の隠蔽" in docs_content

    def test_versioning_semver(self) -> None:
        """テンプレート仕様-ライブラリ.md: SemVer バージョニング。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "SemVer" in docs_content


class TestGoLibIntegrationTestContent:
    """テンプレート仕様-ライブラリ.md: Go tests/integration_test.go.tera の内容テスト。"""

    def setup_method(self) -> None:
        self.content = (LIB_GO / "tests" / "integration_test.go.tera").read_text(encoding="utf-8")

    def test_package_tests(self) -> None:
        assert "package tests" in self.content

    def test_testing_import(self) -> None:
        assert '"testing"' in self.content

    def test_test_function(self) -> None:
        assert "func Test" in self.content

    def test_testify_assert_import(self) -> None:
        assert "testify/assert" in self.content


class TestRustLibIntegrationTestContent:
    """テンプレート仕様-ライブラリ.md: Rust tests/integration_test.rs.tera の内容テスト。"""

    def setup_method(self) -> None:
        self.content = (LIB_RUST / "tests" / "integration_test.rs.tera").read_text(encoding="utf-8")

    def test_use_crate(self) -> None:
        """テンプレート仕様-ライブラリ.md: クレート名でインポート。"""
        assert "use {{ service_name | snake_case }}" in self.content

    def test_test_attribute(self) -> None:
        assert "#[test]" in self.content

    def test_test_function(self) -> None:
        assert "fn test_" in self.content


class TestTsLibIndexTestContent:
    """テンプレート仕様-ライブラリ.md: TypeScript tests/index.test.ts.tera の内容テスト。"""

    def setup_method(self) -> None:
        self.content = (LIB_TS / "tests" / "index.test.ts.tera").read_text(encoding="utf-8")

    def test_vitest_imports(self) -> None:
        """テンプレート仕様-ライブラリ.md: vitest から describe/it/expect を import。"""
        assert "describe" in self.content
        assert "it" in self.content
        assert "expect" in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_has_test_case(self) -> None:
        assert "should be defined" in self.content or "expect" in self.content


class TestGoLibErrorType:
    """Go ライブラリの AppError 型検証。"""
    def setup_method(self) -> None:
        self.content = (LIB_GO / "{name}.go.tera").read_text(encoding="utf-8")

    def test_app_error_struct(self) -> None:
        assert "AppError" in self.content

    def test_error_interface(self) -> None:
        assert "Error() string" in self.content

    def test_validate_method(self) -> None:
        assert "Validate()" in self.content


class TestRustLibErrorType:
    """Rust ライブラリの LibError 型検証。"""
    def setup_method(self) -> None:
        self.content = (LIB_RUST / "src" / "{module}.rs.tera").read_text(encoding="utf-8")

    def test_lib_error_enum(self) -> None:
        assert "LibError" in self.content

    def test_thiserror_derive(self) -> None:
        assert "thiserror::Error" in self.content

    def test_validate_method(self) -> None:
        assert "validate" in self.content


class TestTsLibErrorType:
    """TypeScript ライブラリの AppError 型検証。"""
    def setup_method(self) -> None:
        self.content = (LIB_TS / "src" / "index.ts.tera").read_text(encoding="utf-8")

    def test_app_error_class(self) -> None:
        assert "AppError" in self.content

    def test_validate_function(self) -> None:
        assert "validate" in self.content


class TestDartLibErrorType:
    """Dart ライブラリの AppException 型検証。"""
    def setup_method(self) -> None:
        self.content = (LIB_DART / "lib" / "src" / "{module}.dart.tera").read_text(encoding="utf-8")

    def test_app_exception_class(self) -> None:
        assert "AppException" in self.content

    def test_validate_method(self) -> None:
        assert "validate" in self.content


# ============================================================================
# Swift テンプレート検証
# ============================================================================


class TestSwiftLibPackageSwiftContent:
    """テンプレート仕様-ライブラリ.md: Swift Package.swift.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_SWIFT / "Package.swift.tera").read_text(encoding="utf-8")

    def test_swift_tools_version(self) -> None:
        assert "swift-tools-version: 6.0" in self.content

    def test_service_name_variable(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_service_name_pascal(self) -> None:
        assert "{{ service_name_pascal }}" in self.content

    def test_swift_language_mode(self) -> None:
        assert "swiftLanguageMode(.v6)" in self.content

    def test_platforms(self) -> None:
        assert ".macOS(.v14)" in self.content
        assert ".iOS(.v17)" in self.content

    def test_no_deprecated_language_version(self) -> None:
        """非推奨の swiftLanguageVersion を使っていないこと。"""
        assert "swiftLanguageVersion" not in self.content


class TestSwiftLibSourceContent:
    """テンプレート仕様-ライブラリ.md: Swift Sources テンプレートの内容検証。"""

    def test_lib_error_exists(self) -> None:
        assert (LIB_SWIFT / "Sources" / "{module}" / "LibError.swift.tera").exists()

    def test_config_exists(self) -> None:
        assert (LIB_SWIFT / "Sources" / "{module}" / "Config.swift.tera").exists()

    def test_client_exists(self) -> None:
        assert (LIB_SWIFT / "Sources" / "{module}" / "Client.swift.tera").exists()

    def test_lib_error_sendable(self) -> None:
        content = (LIB_SWIFT / "Sources" / "{module}" / "LibError.swift.tera").read_text(encoding="utf-8")
        assert "Sendable" in content

    def test_lib_error_enum(self) -> None:
        content = (LIB_SWIFT / "Sources" / "{module}" / "LibError.swift.tera").read_text(encoding="utf-8")
        assert "LibError" in content
        assert "Error" in content

    def test_config_sendable(self) -> None:
        content = (LIB_SWIFT / "Sources" / "{module}" / "Config.swift.tera").read_text(encoding="utf-8")
        assert "Sendable" in content

    def test_config_validate_method(self) -> None:
        content = (LIB_SWIFT / "Sources" / "{module}" / "Config.swift.tera").read_text(encoding="utf-8")
        assert "validate" in content

    def test_client_sendable(self) -> None:
        content = (LIB_SWIFT / "Sources" / "{module}" / "Client.swift.tera").read_text(encoding="utf-8")
        assert "Sendable" in content

    def test_client_service_name_pascal(self) -> None:
        content = (LIB_SWIFT / "Sources" / "{module}" / "Client.swift.tera").read_text(encoding="utf-8")
        assert "{{ service_name_pascal }}" in content


class TestSwiftLibTestContent:
    """テンプレート仕様-ライブラリ.md: Swift テストファイルの検証。"""

    def setup_method(self) -> None:
        self.content = (
            LIB_SWIFT / "Tests" / "{module}_tests" / "ClientTests.swift.tera"
        ).read_text(encoding="utf-8")

    def test_test_file_exists(self) -> None:
        assert (LIB_SWIFT / "Tests" / "{module}_tests" / "ClientTests.swift.tera").exists()

    def test_readme_exists(self) -> None:
        assert (LIB_SWIFT / "README.md.tera").exists()

    def test_uses_swift_testing_framework(self) -> None:
        """Swift Testing フレームワーク (@Suite, @Test, #expect) を使用していること。"""
        assert "import Testing" in self.content
        assert "@Suite" in self.content
        assert "@Test" in self.content
        assert "#expect" in self.content

    def test_testable_import(self) -> None:
        assert "@testable import K1s0" in self.content

    def test_service_name_pascal_in_test(self) -> None:
        assert "{{ service_name_pascal }}" in self.content


class TestSwiftLibErrorType:
    """Swift ライブラリの LibError 型検証。"""

    def setup_method(self) -> None:
        self.content = (LIB_SWIFT / "Sources" / "{module}" / "LibError.swift.tera").read_text(encoding="utf-8")

    def test_lib_error_enum(self) -> None:
        assert "LibError" in self.content

    def test_error_conformance(self) -> None:
        assert "Error" in self.content

    def test_sendable_conformance(self) -> None:
        assert "Sendable" in self.content

    def test_custom_string_description(self) -> None:
        assert "description" in self.content


class TestSwiftLibLintTools:
    """テンプレート仕様-ライブラリ.md: Swift の swift-format 検証。"""

    def test_docs_mention_swift_format(self) -> None:
        """テンプレート仕様-ライブラリ.md: swift-format が記載されている。"""
        docs_content = (ROOT / "docs" / "テンプレート仕様-ライブラリ.md").read_text(encoding="utf-8")
        assert "swift-format" in docs_content

    def test_swift_format_config_exists(self) -> None:
        """.swift-format 設定ファイルが存在すること。"""
        swift_libs = ROOT / "regions" / "system" / "library" / "swift"
        assert (swift_libs / ".swift-format").exists()
