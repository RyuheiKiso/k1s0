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
