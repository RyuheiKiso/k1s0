"""テンプレートパイプライン E2E テスト。

仕様書のコマンドマトリクスとソースコードの整合性、
および determine_post_commands の出力を検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"
CLI_SRC = ROOT / "CLI" / "crates" / "k1s0-cli" / "src"
CORE_SRC = ROOT / "CLI" / "crates" / "k1s0-core" / "src"
PIPELINE_SPEC = DOCS / "テンプレート仕様-コード生成パイプライン.md"


class TestPipelineSpecExists:
    """パイプライン仕様書の存在確認。"""

    def test_pipeline_spec_exists(self) -> None:
        assert PIPELINE_SPEC.exists(), "テンプレート仕様-コード生成パイプライン.md が存在しません"


class TestTemplateDirectoryIntegrity:
    """テンプレートディレクトリの整合性検証。"""

    @pytest.mark.parametrize(
        "kind,lang",
        [
            ("server", "go"),
            ("server", "rust"),
            ("bff", "go"),
            ("bff", "rust"),
            ("client", "react"),
            ("client", "flutter"),
            ("library", "go"),
            ("library", "rust"),
        ],
    )
    def test_kind_lang_dir_exists(self, kind: str, lang: str) -> None:
        path = TEMPLATES / kind / lang
        assert path.exists(), f"テンプレートディレクトリ {kind}/{lang} が存在しません"

    @pytest.mark.parametrize(
        "flat_kind",
        [
            "cicd",
            "helm",
            "terraform",
            "docker-compose",
            "devcontainer",
            "service-mesh",
            "kong",
            "keycloak",
            "observability",
        ],
    )
    def test_flat_kind_dir_exists(self, flat_kind: str) -> None:
        path = TEMPLATES / flat_kind
        assert path.exists(), f"テンプレートディレクトリ {flat_kind} が存在しません"


class TestFlatKindsConsistency:
    """flat_kinds 配列とテンプレートディレクトリの整合性検証。"""

    def setup_method(self) -> None:
        mod_rs = CORE_SRC / "template" / "mod.rs"
        assert mod_rs.exists()
        self.mod_content = mod_rs.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "kind",
        [
            "cicd",
            "helm",
            "terraform",
            "docker-compose",
            "devcontainer",
            "service-mesh",
            "kong",
            "keycloak",
            "observability",
        ],
    )
    def test_flat_kind_in_source(self, kind: str) -> None:
        assert (
            f'"{kind}"' in self.mod_content
        ), f"flat_kinds に '{kind}' が含まれていません"


class TestPostCommandsInSpec:
    """仕様書に後処理コマンドが記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = PIPELINE_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "command",
        [
            "go mod tidy",
            "cargo check",
            "npm install",
            "flutter pub get",
        ],
    )
    def test_post_command_documented(self, command: str) -> None:
        assert command in self.content, f"後処理コマンド '{command}' が仕様書に記載されていません"


class TestSpecMentionsNewKinds:
    """仕様書が新規 kind を参照しているかの検証。"""

    def setup_method(self) -> None:
        self.content = PIPELINE_SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "kind",
        [
            "server",
            "database",
        ],
    )
    def test_kind_mentioned_in_spec(self, kind: str) -> None:
        assert kind in self.content, f"kind '{kind}' が仕様書に記載されていません"
