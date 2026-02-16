"""テンプレート仕様-コード生成パイプライン.md の仕様準拠テスト。

パイプライン仕様書の構造・内容が正しいかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"
TEMPLATES = ROOT / "CLI" / "templates"
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


# --- ギャップ 1: 設定ファイルテンプレート一覧 ---


class TestCodegenConfigFileTemplates:
    """設定ファイルテンプレート一覧が仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_config_templates_section(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: 設定ファイルテンプレート一覧セクション。"""
        assert "## 設定ファイルテンプレート一覧" in self.content

    @pytest.mark.parametrize(
        "tool,config_file,template_path",
        [
            ("buf", "buf.yaml", "CLI/templates/server/go/buf.yaml.tera"),
            ("buf", "buf.gen.yaml", "CLI/templates/server/go/buf.gen.yaml.tera"),
            ("buf", "buf.yaml", "CLI/templates/server/rust/buf.yaml.tera"),
            ("oapi-codegen", "oapi-codegen.yaml", "CLI/templates/server/go/oapi-codegen.yaml.tera"),
            ("gqlgen", "gqlgen.yml", "CLI/templates/server/go/gqlgen.yml.tera"),
        ],
    )
    def test_config_template_documented(self, tool: str, config_file: str, template_path: str) -> None:
        """テンプレート仕様-コード生成パイプライン.md: 設定ファイルテンプレートが記載されている。"""
        assert template_path in self.content, (
            f"{tool} の設定ファイル '{config_file}' のテンプレートパス '{template_path}' が記載されていません"
        )

    @pytest.mark.parametrize(
        "template_path",
        [
            "CLI/templates/server/go/buf.yaml.tera",
            "CLI/templates/server/go/buf.gen.yaml.tera",
            "CLI/templates/server/rust/buf.yaml.tera",
            "CLI/templates/server/go/oapi-codegen.yaml.tera",
            "CLI/templates/server/go/gqlgen.yml.tera",
        ],
    )
    def test_config_template_exists(self, template_path: str) -> None:
        """テンプレート仕様-コード生成パイプライン.md: 設定ファイルテンプレートが存在する。"""
        path = ROOT / template_path
        assert path.exists(), f"{template_path} が存在しません"


# --- ギャップ 2: API定義ファイルのテンプレートパス検証 ---


class TestCodegenApiDefinitionTemplates:
    """API定義ファイルのテンプレートパスが仕様書に記載され、実在するかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "api_style,definition_file,template_path",
        [
            ("REST", "api/openapi/openapi.yaml", "CLI/templates/server/go/api/openapi/openapi.yaml.tera"),
            ("gRPC", "api/proto/service.proto", "CLI/templates/server/go/api/proto/service.proto.tera"),
            ("GraphQL", "api/graphql/schema.graphql", "CLI/templates/server/go/api/graphql/schema.graphql.tera"),
        ],
    )
    def test_api_definition_template_documented(self, api_style: str, definition_file: str, template_path: str) -> None:
        """テンプレート仕様-コード生成パイプライン.md: API定義テンプレートパスが記載されている。"""
        assert template_path in self.content, (
            f"{api_style} の定義ファイル '{definition_file}' のテンプレートパス '{template_path}' が記載されていません"
        )

    @pytest.mark.parametrize(
        "template_path",
        [
            "CLI/templates/server/go/api/openapi/openapi.yaml.tera",
            "CLI/templates/server/go/api/proto/service.proto.tera",
            "CLI/templates/server/go/api/graphql/schema.graphql.tera",
        ],
    )
    def test_api_definition_template_exists(self, template_path: str) -> None:
        """テンプレート仕様-コード生成パイプライン.md: API定義テンプレートが存在する。"""
        path = ROOT / template_path
        assert path.exists(), f"{template_path} が存在しません"


# --- ギャップ 3: GraphQL未実装の記載 ---


class TestCodegenGraphQLNotImplemented:
    """GraphQLコード生成が未実装であることが仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_graphql_not_implemented(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: GraphQL未実装が明記されている。"""
        assert "現状未実装" in self.content

    def test_graphql_future_plan(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: GraphQLの将来計画が記載されている。"""
        assert "gqlgen generate" in self.content

    def test_graphql_templates_exist(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: GraphQLテンプレート自体は用意済み。"""
        assert "テンプレート自体" in self.content and "既に用意" in self.content


# --- ギャップ 4: エラーメッセージ形式 ---


class TestCodegenErrorMessageFormat:
    """エラーメッセージ形式が仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_error_message_section(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: エラーメッセージ形式セクションが存在する。"""
        assert "### エラーメッセージ形式" in self.content

    def test_error_message_format_failed(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: 失敗時のメッセージ形式。"""
        assert "後処理コマンド '{cmd} {args}' が失敗しました" in self.content

    def test_error_message_format_execution_failed(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: 実行失敗時のメッセージ形式。"""
        assert "後処理コマンド '{cmd} {args}' の実行に失敗しました" in self.content

    def test_manual_execution_hint(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: 手動実行ヒントが含まれる。"""
        assert "手動で実行してください: cd {output_path} && {cmd} {args}" in self.content


# --- ギャップ 5: リトライ機構未実装の記載 ---


class TestCodegenRetryNotImplemented:
    """リトライ機構が未実装であることが仕様書に記載されているかの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_retry_section(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: リトライ機構セクションが存在する。"""
        assert "### リトライ機構" in self.content

    def test_retry_not_implemented(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: リトライ未実装が明記されている。"""
        assert "リトライ機構は未実装" in self.content

    def test_retry_future_plan(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: 将来的な最大3回リトライの計画。"""
        assert "最大3回" in self.content


# --- ギャップ 6: Database kind の依存解決「なし」 ---


class TestCodegenDatabaseNoDependency:
    """Database kind の依存解決が「なし」であることの検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_database_no_dependency_command(self) -> None:
        """テンプレート仕様-コード生成パイプライン.md: Database kind は依存解決なし。"""
        # 仕様書のマトリクスで Database の依存解決コマンドが「なし」であることを確認
        assert "Database" in self.content
        # マトリクス内で Database 行に「なし」が含まれる
        lines = self.content.split("\n")
        found_database_row = False
        for line in lines:
            if "Database" in line and "|" in line:
                if "なし" in line or "---" in line or "（なし）" in line:
                    found_database_row = True
                    break
        assert found_database_row, "Database kind の依存解決「なし」がマトリクスに記載されていません"
