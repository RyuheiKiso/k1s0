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
