"""コーディング規約.md の仕様準拠テスト。

linter/formatter 設定ファイルの内容がドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]


class TestGolangciLintConfig:
    """コーディング規約.md: .golangci.yaml の検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / ".golangci.yaml"
        assert config_path.exists(), ".golangci.yaml が存在しません"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_timeout(self) -> None:
        assert self.config["run"]["timeout"] == "5m"

    def test_enabled_linters(self) -> None:
        expected = [
            "errcheck",
            "govet",
            "staticcheck",
            "unused",
            "gosimple",
            "ineffassign",
            "misspell",
            "gocyclo",
            "gocritic",
            "revive",
            "gosec",
        ]
        assert self.config["linters"]["enable"] == expected

    def test_gocyclo_complexity(self) -> None:
        assert self.config["linters-settings"]["gocyclo"]["min-complexity"] == 15

    def test_revive_rules(self) -> None:
        revive_rules = self.config["linters-settings"]["revive"]["rules"]
        rule_names = [r["name"] for r in revive_rules]
        assert "exported" in rule_names
        assert "var-naming" in rule_names
        assert "unexported-return" in rule_names

    def test_gosec_includes(self) -> None:
        includes = self.config["linters-settings"]["gosec"]["includes"]
        expected_rules = [
            "G101", "G201", "G301", "G302", "G303", "G304", "G305",
            "G306", "G307", "G401", "G402", "G403", "G404", "G405",
            "G501", "G502", "G503", "G504", "G505",
        ]
        for rule in expected_rules:
            assert rule in includes, f"gosec ルール {rule} が includes に含まれていません"

    def test_gosec_excludes(self) -> None:
        excludes = self.config["linters-settings"]["gosec"]["excludes"]
        assert "G104" in excludes
        assert "G114" in excludes


class TestRustfmtConfig:
    """コーディング規約.md: rustfmt.toml の検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / "rustfmt.toml"
        assert config_path.exists(), "rustfmt.toml が存在しません"
        self.content = config_path.read_text()

    def test_edition(self) -> None:
        assert 'edition = "2021"' in self.content

    def test_max_width(self) -> None:
        assert "max_width = 100" in self.content

    def test_tab_spaces(self) -> None:
        assert "tab_spaces = 4" in self.content

    def test_field_init_shorthand(self) -> None:
        assert "use_field_init_shorthand = true" in self.content


class TestRuffConfig:
    """コーディング規約.md: ruff.toml の検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / "ruff.toml"
        assert config_path.exists(), "ruff.toml が存在しません"
        self.content = config_path.read_text()

    def test_line_length(self) -> None:
        assert "line-length = 100" in self.content

    def test_target_version(self) -> None:
        assert 'target-version = "py312"' in self.content

    def test_lint_select(self) -> None:
        assert 'select = ["E", "F", "I", "UP", "B", "SIM"]' in self.content


class TestMypyConfig:
    """コーディング規約.md: mypy.ini の検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / "mypy.ini"
        assert config_path.exists(), "mypy.ini が存在しません"
        self.content = config_path.read_text()

    def test_python_version(self) -> None:
        assert "python_version = 3.12" in self.content

    def test_strict(self) -> None:
        assert "strict = true" in self.content

    def test_warn_return_any(self) -> None:
        assert "warn_return_any = true" in self.content

    def test_warn_unused_configs(self) -> None:
        assert "warn_unused_configs = true" in self.content


class TestPreCommitConfig:
    """コーディング規約.md: .pre-commit-config.yaml の検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / ".pre-commit-config.yaml"
        assert config_path.exists(), ".pre-commit-config.yaml が存在しません"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_hooks_exist(self) -> None:
        hooks = self.config["repos"][0]["hooks"]
        hook_ids = [h["id"] for h in hooks]
        expected = [
            "go-lint",
            "rust-fmt",
            "rust-clippy",
            "ts-lint",
            "prettier",
            "dart-analyze",
            "dart-format",
            "python-lint",
        ]
        for hook_id in expected:
            assert hook_id in hook_ids, f"pre-commit フック '{hook_id}' が存在しません"


class TestClippyLints:
    """コーディング規約.md: CLI/Cargo.toml の clippy lints セクション検証。"""

    def test_clippy_config(self) -> None:
        cargo_toml = ROOT / "CLI" / "Cargo.toml"
        assert cargo_toml.exists()
        content = cargo_toml.read_text()
        assert "[lints.clippy]" in content
        assert "pedantic" in content
        assert "module_name_repetitions" in content
        assert "must_use_candidate" in content


class TestEslintConfigRules:
    """コーディング規約.md: eslint.config.mjs 具体ルールの検証。"""

    def setup_method(self) -> None:
        path = ROOT / "eslint.config.mjs"
        assert path.exists(), "eslint.config.mjs が存在しません"
        self.content = path.read_text(encoding="utf-8")

    def test_react_hooks_rules_of_hooks(self) -> None:
        """コーディング規約.md: react-hooks/rules-of-hooks: error。"""
        assert "react-hooks/rules-of-hooks" in self.content

    def test_react_hooks_exhaustive_deps(self) -> None:
        """コーディング規約.md: react-hooks/exhaustive-deps: warn。"""
        assert "react-hooks/exhaustive-deps" in self.content

    def test_import_order_rule(self) -> None:
        """コーディング規約.md: import/order ルールが設定されている。"""
        assert "import/order" in self.content

    def test_no_unused_vars_rule(self) -> None:
        """コーディング規約.md: @typescript-eslint/no-unused-vars: error。"""
        assert "@typescript-eslint/no-unused-vars" in self.content

    def test_no_floating_promises_rule(self) -> None:
        """コーディング規約.md: @typescript-eslint/no-floating-promises: error。"""
        assert "@typescript-eslint/no-floating-promises" in self.content

    def test_strict_type_checked(self) -> None:
        """コーディング規約.md: strictTypeChecked が有効。"""
        assert "strictTypeChecked" in self.content


class TestPrettierrcConfig:
    """コーディング規約.md: .prettierrc 具体設定の検証。"""

    def setup_method(self) -> None:
        import json
        path = ROOT / ".prettierrc"
        assert path.exists(), ".prettierrc が存在しません"
        self.content = path.read_text(encoding="utf-8")
        self.config = json.loads(self.content)

    def test_semi_true(self) -> None:
        """コーディング規約.md: semi: true。"""
        assert self.config["semi"] is True

    def test_single_quote_true(self) -> None:
        """コーディング規約.md: singleQuote: true。"""
        assert self.config["singleQuote"] is True

    def test_trailing_comma_all(self) -> None:
        """コーディング規約.md: trailingComma: all。"""
        assert self.config["trailingComma"] == "all"

    def test_print_width_100(self) -> None:
        """コーディング規約.md: printWidth: 100。"""
        assert self.config["printWidth"] == 100

    def test_tab_width_2(self) -> None:
        """コーディング規約.md: tabWidth: 2。"""
        assert self.config["tabWidth"] == 2


class TestDartAnalysisOptionsRules:
    """コーディング規約.md: analysis_options.yaml 具体ルールの検証。"""

    def setup_method(self) -> None:
        path = ROOT / "analysis_options.yaml"
        assert path.exists(), "analysis_options.yaml が存在しません"
        self.content = path.read_text(encoding="utf-8")
        self.config = yaml.safe_load(self.content)

    def test_prefer_const_constructors(self) -> None:
        """コーディング規約.md: prefer_const_constructors ルール。"""
        assert "prefer_const_constructors" in self.config["linter"]["rules"]

    def test_prefer_const_declarations(self) -> None:
        """コーディング規約.md: prefer_const_declarations ルール。"""
        assert "prefer_const_declarations" in self.config["linter"]["rules"]

    def test_avoid_print(self) -> None:
        """コーディング規約.md: avoid_print ルール。"""
        assert "avoid_print" in self.config["linter"]["rules"]

    def test_prefer_single_quotes(self) -> None:
        """コーディング規約.md: prefer_single_quotes ルール。"""
        assert "prefer_single_quotes" in self.config["linter"]["rules"]

    def test_flutter_lints_include(self) -> None:
        """コーディング規約.md: flutter_lints パッケージを include。"""
        assert "flutter_lints" in self.content


class TestVitestConfigTemplate:
    """コーディング規約.md: vitest.config.ts テンプレート仕様の検証。"""

    def test_vitest_spec_in_doc(self) -> None:
        """コーディング規約.md: vitest.config.ts 仕様がドキュメントに記載。"""
        doc = ROOT / "docs" / "コーディング規約.md"
        content = doc.read_text(encoding="utf-8")
        assert "vitest" in content.lower()
        assert "defineConfig" in content

    def test_vitest_globals_true_in_doc(self) -> None:
        """コーディング規約.md: globals: true が仕様に記載。"""
        doc = ROOT / "docs" / "コーディング規約.md"
        content = doc.read_text(encoding="utf-8")
        assert "globals: true" in content

    def test_vitest_jsdom_environment_in_doc(self) -> None:
        """コーディング規約.md: environment: jsdom が仕様に記載。"""
        doc = ROOT / "docs" / "コーディング規約.md"
        content = doc.read_text(encoding="utf-8")
        assert "jsdom" in content

    def test_vitest_v8_coverage_in_doc(self) -> None:
        """コーディング規約.md: coverage provider v8 が仕様に記載。"""
        doc = ROOT / "docs" / "コーディング規約.md"
        content = doc.read_text(encoding="utf-8")
        assert "v8" in content


class TestGoTestToolsConfig:
    """コーディング規約.md: Go テストツール（testify, gomock）の検証。"""

    def test_testify_in_doc(self) -> None:
        """コーディング規約.md: testify がドキュメントに記載。"""
        doc = ROOT / "docs" / "コーディング規約.md"
        content = doc.read_text(encoding="utf-8")
        assert "testify" in content

    def test_gomock_in_doc(self) -> None:
        """コーディング規約.md: gomock がドキュメントに記載。"""
        doc = ROOT / "docs" / "コーディング規約.md"
        content = doc.read_text(encoding="utf-8")
        assert "gomock" in content

    def test_mockall_in_doc(self) -> None:
        """コーディング規約.md: Rust mockall がドキュメントに記載。"""
        doc = ROOT / "docs" / "コーディング規約.md"
        content = doc.read_text(encoding="utf-8")
        assert "mockall" in content


class TestNamingConventionTable:
    """コーディング規約.md: 命名規則テーブル検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_go_camel_case(self) -> None:
        assert "camelCase" in self.content

    def test_rust_snake_case(self) -> None:
        assert "snake_case" in self.content

    def test_pascal_case(self) -> None:
        assert "PascalCase" in self.content

    def test_screaming_snake_case(self) -> None:
        assert "SCREAMING_SNAKE_CASE" in self.content

    @pytest.mark.parametrize(
        "lang",
        ["Go", "Rust", "TypeScript", "Dart", "Python"],
    )
    def test_language_in_naming_table(self, lang: str) -> None:
        """コーディング規約.md: 命名規則テーブルに各言語が記載。"""
        assert lang in self.content


class TestAutoFormatOnSave:
    """コーディング規約.md: 保存時自動フォーマット検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_auto_format_documented(self) -> None:
        """コーディング規約.md: 保存時に自動実行が記載。"""
        assert "保存時に自動実行" in self.content


class TestNolintReasonRequired:
    """コーディング規約.md: nolint 理由明記検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_nolint_reason_documented(self) -> None:
        """コーディング規約.md: 抑制コメントには理由を明記すると記載。"""
        assert "理由を明記" in self.content

    def test_nolint_example_documented(self) -> None:
        """コーディング規約.md: nolint / allow / eslint-disable の例が記載。"""
        assert "nolint" in self.content
        assert "#[allow]" in self.content
        assert "eslint-disable" in self.content


class TestPreCommitHookDetails:
    """コーディング規約.md: pre-commit フック entry コマンドと files パターン検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / ".pre-commit-config.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)
        self.hooks = self.config["repos"][0]["hooks"]
        self.hooks_by_id = {h["id"]: h for h in self.hooks}

    def test_go_lint_entry(self) -> None:
        """コーディング規約.md: go-lint の entry が golangci-lint run。"""
        assert self.hooks_by_id["go-lint"]["entry"] == "golangci-lint run"

    def test_rust_fmt_entry(self) -> None:
        """コーディング規約.md: rust-fmt の entry が cargo fmt -- --check。"""
        assert self.hooks_by_id["rust-fmt"]["entry"] == "cargo fmt -- --check"

    def test_rust_clippy_entry(self) -> None:
        """コーディング規約.md: rust-clippy の entry が cargo clippy -- -D warnings。"""
        assert self.hooks_by_id["rust-clippy"]["entry"] == "cargo clippy -- -D warnings"

    def test_ts_lint_entry(self) -> None:
        """コーディング規約.md: ts-lint の entry が npx eslint。"""
        assert self.hooks_by_id["ts-lint"]["entry"] == "npx eslint"

    def test_prettier_entry(self) -> None:
        """コーディング規約.md: prettier の entry が npx prettier --check。"""
        assert self.hooks_by_id["prettier"]["entry"] == "npx prettier --check"

    def test_dart_analyze_entry(self) -> None:
        """コーディング規約.md: dart-analyze の entry が dart analyze --fatal-infos。"""
        assert self.hooks_by_id["dart-analyze"]["entry"] == "dart analyze --fatal-infos"

    def test_dart_format_entry(self) -> None:
        """コーディング規約.md: dart-format の entry が dart format --set-exit-if-changed。"""
        assert self.hooks_by_id["dart-format"]["entry"] == "dart format --set-exit-if-changed"

    def test_python_lint_entry(self) -> None:
        """コーディング規約.md: python-lint の entry が ruff check。"""
        assert self.hooks_by_id["python-lint"]["entry"] == "ruff check"


class TestPreCommitHookFilesPattern:
    """コーディング規約.md: pre-commit フック files パターン検証。"""

    def setup_method(self) -> None:
        config_path = ROOT / ".pre-commit-config.yaml"
        with open(config_path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)
        self.hooks = self.config["repos"][0]["hooks"]
        self.hooks_by_id = {h["id"]: h for h in self.hooks}

    def test_go_lint_files_pattern(self) -> None:
        assert self.hooks_by_id["go-lint"]["files"] == r"\.go$"

    def test_rust_fmt_files_pattern(self) -> None:
        assert self.hooks_by_id["rust-fmt"]["files"] == r"\.rs$"

    def test_ts_lint_files_pattern(self) -> None:
        assert self.hooks_by_id["ts-lint"]["files"] == r"\.[tj]sx?$"

    def test_dart_analyze_files_pattern(self) -> None:
        assert self.hooks_by_id["dart-analyze"]["files"] == r"\.dart$"

    def test_python_lint_files_pattern(self) -> None:
        assert self.hooks_by_id["python-lint"]["files"] == r"\.py$"


class TestRustTokioDevDependencies:
    """コーディング規約.md: Rust tokio dev-dependencies 検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_tokio_documented(self) -> None:
        """コーディング規約.md: tokio が dev-dependencies に記載。"""
        assert "tokio" in self.content
        assert "dev-dependencies" in self.content

    def test_tokio_test_util(self) -> None:
        """コーディング規約.md: tokio test-util が記載。"""
        assert "test-util" in self.content


class TestTypeScriptTestingLibrary:
    """コーディング規約.md: TypeScript Testing Library 検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_testing_library_documented(self) -> None:
        """コーディング規約.md: @testing-library/react が記載。"""
        assert "@testing-library/react" in self.content

    def test_jest_dom_documented(self) -> None:
        """コーディング規約.md: @testing-library/jest-dom が記載。"""
        assert "@testing-library/jest-dom" in self.content


class TestTypeScriptMSW:
    """コーディング規約.md: TypeScript MSW 検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_msw_documented(self) -> None:
        """コーディング規約.md: MSW が記載。"""
        assert "msw" in self.content.lower() or "MSW" in self.content


class TestDartMocktail:
    """コーディング規約.md: Dart mocktail 検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_mocktail_documented(self) -> None:
        """コーディング規約.md: mocktail が記載。"""
        assert "mocktail" in self.content


class TestDartFlutterTest:
    """コーディング規約.md: Dart flutter_test 検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_flutter_test_documented(self) -> None:
        """コーディング規約.md: flutter_test が記載。"""
        assert "flutter_test" in self.content


class TestEslintTestConfig:
    """コーディング規約.md: ESLint テスト用設定検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_test_config_documented(self) -> None:
        """コーディング規約.md: テスト用 ESLint 設定が記載。"""
        assert "tests/**" in self.content
        assert "describe" in self.content
        assert "readonly" in self.content


class TestMSWHandlerPattern:
    """コーディング規約.md: MSW ハンドラーパターン検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_msw_handler_documented(self) -> None:
        """コーディング規約.md: MSW ハンドラーパターンが記載。"""
        assert "handlers" in self.content
        assert "http.get" in self.content or "HttpResponse" in self.content

    def test_msw_setup_server(self) -> None:
        """コーディング規約.md: setupServer が記載。"""
        assert "setupServer" in self.content


class TestTestcontainersPattern:
    """コーディング規約.md: testcontainers パターン検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_testcontainers_documented(self) -> None:
        """コーディング規約.md: testcontainers が記載。"""
        assert "testcontainers" in self.content


class TestTestFileNamingConvention:
    """コーディング規約.md: テストファイル命名規則検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_go_test_naming(self) -> None:
        assert "*_test.go" in self.content

    def test_rust_test_naming(self) -> None:
        assert "#[cfg(test)]" in self.content

    def test_ts_test_naming(self) -> None:
        assert "*.test.ts" in self.content or ".test.ts" in self.content

    def test_dart_test_naming(self) -> None:
        assert "*_test.dart" in self.content

    def test_python_test_naming(self) -> None:
        assert "test_*.py" in self.content


class TestMockAutoGeneration:
    """コーディング規約.md: モック自動生成検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_auto_mock_documented(self) -> None:
        """コーディング規約.md: モックは自動生成を基本と記載。"""
        assert "自動生成" in self.content


class TestGoGenerateBuildRs:
    """コーディング規約.md: go generate / build.rs 検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_go_generate_documented(self) -> None:
        """コーディング規約.md: go generate が記載。"""
        assert "go generate" in self.content

    def test_build_rs_documented(self) -> None:
        """コーディング規約.md: build.rs が記載。"""
        assert "build.rs" in self.content


class TestCodeReviewGuidelines:
    """コーディング規約.md: コードレビューガイドライン検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "review_point",
        [
            "機能要件",
            "Tier 依存ルール",
            "エラーハンドリング",
            "セキュリティ",
            "テスト",
            "パフォーマンス",
        ],
    )
    def test_review_point_documented(self, review_point: str) -> None:
        """コーディング規約.md: レビュー観点が記載。"""
        assert review_point in self.content


class TestReviewProcess:
    """コーディング規約.md: レビュープロセス検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_pr_approve_requirement(self) -> None:
        """コーディング規約.md: PR は最低 1 名の Approve が必要。"""
        assert "1 名" in self.content or "1名" in self.content

    def test_system_tier_two_approves(self) -> None:
        """コーディング規約.md: system Tier は 2 名の Approve が必要。"""
        assert "2 名" in self.content or "2名" in self.content


class TestGomockGoGeneratePattern:
    """コーディング規約.md: gomock go:generate パターン検証。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_go_generate_mockgen_pattern(self) -> None:
        """コーディング規約.md: go:generate mockgen パターンが記載。"""
        assert "go:generate" in self.content
        assert "mockgen" in self.content


class TestEslintPluginImport:
    """コーディング規約.md: eslint-plugin-import 検証。"""

    def setup_method(self) -> None:
        self.eslint_content = (ROOT / "eslint.config.mjs").read_text(encoding="utf-8")
        self.doc_content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_import_plugin_in_eslint(self) -> None:
        """eslint.config.mjs に eslint-plugin-import が含まれる。"""
        assert "eslint-plugin-import" in self.eslint_content

    def test_import_plugin_in_doc(self) -> None:
        """コーディング規約.md に import プラグインが記載。"""
        assert "eslint-plugin-import" in self.doc_content


class TestExplicitFunctionReturnTypeOff:
    """コーディング規約.md: explicit-function-return-type: off 検証。"""

    def setup_method(self) -> None:
        self.eslint_content = (ROOT / "eslint.config.mjs").read_text(encoding="utf-8")
        self.doc_content = (ROOT / "docs" / "コーディング規約.md").read_text(encoding="utf-8")

    def test_explicit_return_type_off_in_eslint(self) -> None:
        """eslint.config.mjs に explicit-function-return-type: off が設定。"""
        assert "explicit-function-return-type" in self.eslint_content
        assert '"off"' in self.eslint_content

    def test_explicit_return_type_off_in_doc(self) -> None:
        """コーディング規約.md に explicit-function-return-type: off が記載。"""
        assert "explicit-function-return-type" in self.doc_content
