"""テンプレート仕様-コード生成パイプライン.md の仕様準拠テスト。

パイプライン仕様書の構造・内容が正しいかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-コード生成パイプライン.md"


class TestCodegenPipelineSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-コード生成パイプライン.md が存在しません"


class TestCodegenPipelineSections:
    """仕様書に主要セクションが存在するかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "section",
        [
            "## 概要",
            "## パイプライン全体フロー",
            "## 実行条件マトリクス",
            "## 依存解決パイプライン",
            "## コード生成パイプライン",
            "## DB マイグレーション初期化",
            "## エラーハンドリング仕様",
            "## 関連ドキュメント",
        ],
    )
    def test_section_exists(self, section: str) -> None:
        assert section in self.content, f"セクション '{section}' が仕様書に存在しません"


class TestCodegenPipelinePostCommands:
    """後処理コマンドが仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "command",
        [
            "go mod tidy",
            "cargo check",
            "npm install",
            "flutter pub get",
            "buf generate",
            "oapi-codegen",
            "cargo xtask codegen",
            "sqlx database create",
        ],
    )
    def test_post_command_documented(self, command: str) -> None:
        assert command in self.content, f"後処理コマンド '{command}' が仕様書に記載されていません"


class TestCodegenPipelineMatrix:
    """実行条件マトリクスが正しい組み合わせを含むかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "kind,language",
        [
            ("Server", "Go"),
            ("Server", "Rust"),
            ("Client", "React"),
            ("Client", "Flutter"),
            ("Library", "Go"),
            ("Library", "Rust"),
            ("Library", "TypeScript"),
            ("Library", "Dart"),
        ],
    )
    def test_dependency_matrix_entry(self, kind: str, language: str) -> None:
        assert kind in self.content, f"kind '{kind}' がマトリクスに存在しません"
        assert language in self.content, f"language '{language}' がマトリクスに存在しません"

    @pytest.mark.parametrize(
        "api_style,language",
        [
            ("gRPC", "Go"),
            ("gRPC", "Rust"),
            ("REST", "Go"),
            ("REST", "Rust"),
        ],
    )
    def test_codegen_matrix_entry(self, api_style: str, language: str) -> None:
        assert api_style in self.content, f"api_style '{api_style}' がマトリクスに存在しません"


class TestCodegenPipelineErrorHandling:
    """エラーハンドリングの「best-effort」方式が記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_best_effort_documented(self) -> None:
        assert "best-effort" in self.content, "best-effort 方式が仕様書に記載されていません"

    def test_no_rollback_policy(self) -> None:
        assert "ロールバックは行わない" in self.content

    def test_continue_on_failure(self) -> None:
        assert "後続コマンドの実行は継続する" in self.content

    def test_manual_execution_guidance(self) -> None:
        assert "手動で実行してください" in self.content


class TestCodegenPipelineFunctionMapping:
    """仕様書の関数マッピングが正しく記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_execute_generate_function(self) -> None:
        assert "execute_generate_with_config" in self.content

    def test_try_generate_function(self) -> None:
        assert "try_generate_from_templates" in self.content

    def test_run_post_processing_function(self) -> None:
        assert "run_post_processing" in self.content

    def test_determine_post_commands_function(self) -> None:
        assert "determine_post_commands" in self.content


class TestCodegenPipelineImplementationExists:
    """パイプライン実装ファイルが存在し、仕様書の関数を含むかの検証。"""

    def setup_method(self) -> None:
        self.generate_rs = (ROOT / "CLI" / "src" / "commands" / "generate.rs").read_text(encoding="utf-8")

    def test_generate_rs_exists(self) -> None:
        assert (ROOT / "CLI" / "src" / "commands" / "generate.rs").exists()

    def test_determine_post_commands_exists(self) -> None:
        assert "fn determine_post_commands" in self.generate_rs

    def test_run_post_processing_exists(self) -> None:
        assert "fn run_post_processing" in self.generate_rs

    def test_go_mod_tidy_in_pipeline(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: Go依存解決。"""
        assert '"mod"' in self.generate_rs and '"tidy"' in self.generate_rs

    def test_cargo_check_in_pipeline(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: Rust依存解決。"""
        assert '"cargo"' in self.generate_rs and '"check"' in self.generate_rs

    def test_npm_install_in_pipeline(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: npm依存解決。"""
        assert '"npm"' in self.generate_rs and '"install"' in self.generate_rs

    def test_flutter_pub_get_in_pipeline(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: Flutter依存解決。"""
        assert '"flutter"' in self.generate_rs and '"pub"' in self.generate_rs

    def test_buf_generate_in_pipeline(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: gRPCコード生成。"""
        assert '"buf"' in self.generate_rs and '"generate"' in self.generate_rs

    def test_oapi_codegen_in_pipeline(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: OpenAPIコード生成。"""
        assert '"oapi-codegen"' in self.generate_rs

    def test_sqlx_database_create_in_pipeline(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: DB初期化。"""
        assert '"sqlx"' in self.generate_rs and '"database"' in self.generate_rs

    def test_pipeline_execution_order(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: パイプライン実行順序。

        依存解決 → コード生成 → DB初期化の順序で実行される。
        """
        content = self.generate_rs
        dep_pos = content.find("言語固有の依存解決") or content.find("mod tidy")
        codegen_pos = content.find("コード生成") if content.find("コード生成") > dep_pos else len(content)
        db_pos = content.find("DB ありの場合") or content.find("sqlx")
        # 依存解決 < コード生成 < DB初期化
        assert dep_pos < codegen_pos, "依存解決はコード生成より先に実行されるべき"
        assert codegen_pos < db_pos, "コード生成はDB初期化より先に実行されるべき"
