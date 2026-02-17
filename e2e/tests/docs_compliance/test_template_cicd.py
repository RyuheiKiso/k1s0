"""テンプレート仕様-CICD.md の仕様準拠テスト。

CICDテンプレート仕様書とテンプレートファイルの検証。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates"
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-CICD.md"
CICD = TEMPLATES / "cicd"


class TestCicdSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-CICD.md が存在しません"


class TestCicdTemplateFilesExist:
    """テンプレートファイルの存在確認。"""

    @pytest.mark.parametrize(
        "template",
        [
            "ci.yaml.tera",
            "deploy.yaml.tera",
        ],
    )
    def test_cicd_template_exists(self, template: str) -> None:
        path = CICD / template
        assert path.exists(), f"cicd/{template} が存在しません"


class TestCicdSpecSections:
    """仕様書の主要セクション存在チェック。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "section",
        [
            "## 概要",
            "## 生成対象",
            "## 配置パス",
            "## テンプレートファイル一覧",
            "## 使用するテンプレート変数",
            "## GitHub Actions / Tera 構文衝突の回避",
            "## CI ワークフロー仕様",
            "## Deploy ワークフロー仕様",
            "## 言語バージョン",
        ],
    )
    def test_section_exists(self, section: str) -> None:
        assert section in self.content, f"セクション '{section}' が仕様書に存在しません"


class TestCicdSpecRawSyntax:
    """GitHub Actions/Tera構文衝突の回避（{% raw %} の記載）検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_raw_block_documented(self) -> None:
        assert "{% raw %}" in self.content, "{% raw %} が仕様書に記載されていません"

    def test_endraw_block_documented(self) -> None:
        assert "{% endraw %}" in self.content, "{% endraw %} が仕様書に記載されていません"


class TestCicdTemplateVariables:
    """テンプレート変数の使用チェック。"""

    @pytest.mark.parametrize(
        "template,variable",
        [
            ("ci.yaml.tera", "{{ service_name }}"),
            ("ci.yaml.tera", "{{ module_path }}"),
            ("deploy.yaml.tera", "{{ service_name }}"),
            ("deploy.yaml.tera", "{{ module_path }}"),
            ("deploy.yaml.tera", "{{ docker_project }}"),
            ("deploy.yaml.tera", "{{ docker_registry }}"),
            ("deploy.yaml.tera", "{{ helm_path }}"),
            ("deploy.yaml.tera", "{{ tier }}"),
        ],
    )
    def test_template_variable_used(self, template: str, variable: str) -> None:
        path = CICD / template
        content = path.read_text(encoding="utf-8")
        assert variable in content, f"cicd/{template} に変数 '{variable}' が含まれていません"


class TestCicdSpecGenerationTarget:
    """生成対象の検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "kind",
        ["server", "client", "library", "database"],
    )
    def test_kind_documented(self, kind: str) -> None:
        assert kind in self.content, f"kind '{kind}' が生成対象に記載されていません"

    def test_deploy_server_only(self) -> None:
        assert "server" in self.content
        assert "Deploy" in self.content


class TestCicdDirectoryStructure:
    """仕様書に定義されたディレクトリ構造との一致検証。"""

    def test_cicd_directory_is_flat(self) -> None:
        """cicd/ 直下に ci.yaml.tera と deploy.yaml.tera がフラットに配置されている。"""
        assert (CICD / "ci.yaml.tera").exists()
        assert (CICD / "deploy.yaml.tera").exists()

    def test_no_language_subdirectories(self) -> None:
        """cicd/ 配下に言語別サブディレクトリが存在しない。"""
        for item in CICD.iterdir():
            assert item.is_file(), f"cicd/ 配下にディレクトリ '{item.name}' が存在します（フラット構造違反）"


class TestCicdTemplateRawBlocks:
    """テンプレートファイル内の {% raw %} ブロック検証。"""

    @pytest.mark.parametrize("template", ["ci.yaml.tera", "deploy.yaml.tera"])
    def test_template_has_raw_blocks(self, template: str) -> None:
        path = CICD / template
        content = path.read_text(encoding="utf-8")
        assert "{% raw %}" in content, f"cicd/{template} に {{% raw %}} ブロックがありません"
        assert "{% endraw %}" in content, f"cicd/{template} に {{% endraw %}} ブロックがありません"


class TestCicdSpecLanguageVersions:
    """言語バージョンの記載チェック。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "tool,version",
        [
            ("Go", "1.23"),
            ("Rust", "1.82"),
            ("Node.js", "22"),
            ("Flutter", "3.24.0"),
            ("Helm", "3.16"),
        ],
    )
    def test_version_documented(self, tool: str, version: str) -> None:
        assert version in self.content, f"{tool} のバージョン '{version}' が仕様書に記載されていません"


class TestCicdCILanguageSteps:
    """テンプレート仕様-CICD.md: CI 言語別ステップ内容の検証。"""

    def setup_method(self) -> None:
        self.ci_content = (CICD / "ci.yaml.tera").read_text(encoding="utf-8")
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_go_lint_step(self) -> None:
        """CI テンプレートに Go lint (golangci-lint) ステップが含まれる。"""
        assert "golangci-lint-action" in self.ci_content

    def test_rust_lint_step(self) -> None:
        """CI テンプレートに Rust lint (clippy + rustfmt) ステップが含まれる。"""
        assert "cargo fmt" in self.ci_content
        assert "cargo clippy" in self.ci_content

    def test_ts_lint_step(self) -> None:
        """CI テンプレートに TypeScript lint (eslint + prettier) ステップが含まれる。"""
        assert "npx eslint" in self.ci_content
        assert "npx prettier" in self.ci_content

    def test_dart_lint_step(self) -> None:
        """CI テンプレートに Dart lint (dart analyze + dart format) ステップが含まれる。"""
        assert "dart analyze" in self.ci_content
        assert "dart format" in self.ci_content


class TestCicdCIPipelineStructure:
    """テンプレート仕様-CICD.md: CI パイプライン構成（lint→test→build→security-scan）検証。"""

    def setup_method(self) -> None:
        self.ci_content = (CICD / "ci.yaml.tera").read_text(encoding="utf-8")
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_pipeline_structure_documented(self) -> None:
        """仕様書に lint → test → build → security-scan 構成が記載。"""
        assert "lint" in self.spec_content
        assert "test" in self.spec_content
        assert "build" in self.spec_content
        assert "security-scan" in self.spec_content

    def test_lint_job_in_template(self) -> None:
        assert "lint:" in self.ci_content

    def test_test_needs_lint_in_template(self) -> None:
        assert "needs: lint" in self.ci_content

    def test_build_needs_test_in_template(self) -> None:
        assert "needs: test" in self.ci_content

    def test_security_scan_needs_build_in_template(self) -> None:
        assert "needs: build" in self.ci_content


class TestCicdGrpcConditionalStep:
    """テンプレート仕様-CICD.md: gRPC 条件ステップ（buf lint）検証。"""

    def setup_method(self) -> None:
        self.ci_content = (CICD / "ci.yaml.tera").read_text(encoding="utf-8")
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_grpc_condition_in_template(self) -> None:
        """CI テンプレートに api_style == grpc の条件分岐がある。"""
        assert 'api_style == "grpc"' in self.ci_content or "grpc" in self.ci_content

    def test_buf_lint_in_template(self) -> None:
        """CI テンプレートに buf lint ステップがある。"""
        assert "buf lint" in self.ci_content

    def test_buf_breaking_in_template(self) -> None:
        """CI テンプレートに buf breaking ステップがある。"""
        assert "buf breaking" in self.ci_content

    def test_proto_lint_job_in_template(self) -> None:
        """CI テンプレートに proto-lint ジョブがある。"""
        assert "proto-lint:" in self.ci_content


class TestCicdDBConditionalStep:
    """テンプレート仕様-CICD.md: DB 条件ステップ（migration-test）検証。"""

    def setup_method(self) -> None:
        self.ci_content = (CICD / "ci.yaml.tera").read_text(encoding="utf-8")

    def test_has_database_condition(self) -> None:
        """CI テンプレートに has_database の条件分岐がある。"""
        assert "has_database" in self.ci_content

    def test_migration_test_job(self) -> None:
        """CI テンプレートに migration-test ジョブがある。"""
        assert "migration-test:" in self.ci_content


class TestCicdDBTypeBranch:
    """テンプレート仕様-CICD.md: DB 種別分岐検証。"""

    def setup_method(self) -> None:
        self.ci_content = (CICD / "ci.yaml.tera").read_text(encoding="utf-8")

    def test_postgresql_branch(self) -> None:
        """CI テンプレートに postgresql の分岐がある。"""
        assert "postgresql" in self.ci_content

    def test_mysql_branch(self) -> None:
        """CI テンプレートに mysql の分岐がある。"""
        assert "mysql" in self.ci_content


class TestCicdSecurityScanStep:
    """テンプレート仕様-CICD.md: セキュリティスキャンステップ検証。"""

    def setup_method(self) -> None:
        self.ci_content = (CICD / "ci.yaml.tera").read_text(encoding="utf-8")

    def test_trivy_action_in_template(self) -> None:
        """CI テンプレートに Trivy アクションがある。"""
        assert "aquasecurity/trivy-action" in self.ci_content

    def test_severity_high_critical(self) -> None:
        """CI テンプレートに HIGH,CRITICAL の severity 設定がある。"""
        assert "HIGH,CRITICAL" in self.ci_content

    def test_security_scan_job(self) -> None:
        """CI テンプレートに security-scan ジョブがある。"""
        assert "security-scan:" in self.ci_content


class TestCicdDeployPipeline:
    """テンプレート仕様-CICD.md: Deploy パイプライン構成検証。"""

    def setup_method(self) -> None:
        self.deploy_content = (CICD / "deploy.yaml.tera").read_text(encoding="utf-8")
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_deploy_pipeline_documented(self) -> None:
        """仕様書に deploy パイプライン構成が記載。"""
        assert "build-and-push" in self.spec_content
        assert "deploy-dev" in self.spec_content
        assert "deploy-staging" in self.spec_content
        assert "deploy-prod" in self.spec_content

    def test_deploy_dev_in_template(self) -> None:
        assert "deploy-dev:" in self.deploy_content

    def test_deploy_staging_in_template(self) -> None:
        assert "deploy-staging:" in self.deploy_content

    def test_deploy_prod_in_template(self) -> None:
        assert "deploy-prod:" in self.deploy_content


class TestCicdDeployCosign:
    """テンプレート仕様-CICD.md: Deploy Cosign 署名・検証検証。"""

    def setup_method(self) -> None:
        self.deploy_content = (CICD / "deploy.yaml.tera").read_text(encoding="utf-8")

    def test_cosign_sign_in_template(self) -> None:
        """Deploy テンプレートに cosign sign がある。"""
        assert "cosign sign" in self.deploy_content

    def test_cosign_verify_in_template(self) -> None:
        """Deploy テンプレートに cosign verify がある。"""
        assert "cosign verify" in self.deploy_content

    def test_cosign_installer_in_template(self) -> None:
        """Deploy テンプレートに cosign-installer がある。"""
        assert "sigstore/cosign-installer" in self.deploy_content


class TestCicdDeployImageTag:
    """テンプレート仕様-CICD.md: Deploy イメージタグ規則検証。"""

    def setup_method(self) -> None:
        self.deploy_content = (CICD / "deploy.yaml.tera").read_text(encoding="utf-8")
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_version_tag_in_template(self) -> None:
        """Deploy テンプレートに version タグがある。"""
        assert "version" in self.deploy_content

    def test_sha_tag_in_template(self) -> None:
        """Deploy テンプレートに sha タグがある。"""
        assert "sha" in self.deploy_content

    def test_latest_tag_in_template(self) -> None:
        """Deploy テンプレートに latest タグがある。"""
        assert "latest" in self.deploy_content

    def test_image_tag_rules_documented(self) -> None:
        """仕様書にイメージタグ規則が記載。"""
        assert "{version}" in self.spec_content
        assert "{version}-{git-sha}" in self.spec_content
        assert "latest" in self.spec_content


class TestCicdDeployEnvValues:
    """テンプレート仕様-CICD.md: Deploy 環境別 values 参照検証。"""

    def setup_method(self) -> None:
        self.deploy_content = (CICD / "deploy.yaml.tera").read_text(encoding="utf-8")

    def test_values_dev_yaml(self) -> None:
        assert "values-dev.yaml" in self.deploy_content

    def test_values_staging_yaml(self) -> None:
        assert "values-staging.yaml" in self.deploy_content

    def test_values_prod_yaml(self) -> None:
        assert "values-prod.yaml" in self.deploy_content


class TestCicdCacheStrategy:
    """テンプレート仕様-CICD.md: キャッシュ戦略記載検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "lang,cache",
        [
            ("Go", "~/go/pkg/mod"),
            ("Rust", "~/.cargo"),
            ("Node", "node_modules/"),
            ("Dart", "~/.pub-cache"),
        ],
    )
    def test_cache_strategy_in_spec(self, lang: str, cache: str) -> None:
        """仕様書にキャッシュ戦略が記載。"""
        assert cache in self.spec_content, f"{lang} のキャッシュ対象 '{cache}' が仕様書に記載されていません"

    def test_docker_cache_gha(self) -> None:
        """仕様書に Docker layer cache (type=gha) が記載。"""
        assert "cache-from: type=gha" in self.spec_content


class TestCicdConditionalGenerationTable:
    """テンプレート仕様-CICD.md: 条件付き生成表検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_language_condition(self) -> None:
        """仕様書に language 条件が記載。"""
        assert "language" in self.spec_content
        assert "`go`" in self.spec_content
        assert "`rust`" in self.spec_content

    def test_framework_condition(self) -> None:
        """仕様書に framework 条件が記載。"""
        assert "framework" in self.spec_content
        assert "`react`" in self.spec_content
        assert "`flutter`" in self.spec_content

    def test_api_style_condition(self) -> None:
        """仕様書に api_style 条件が記載。"""
        assert "api_style" in self.spec_content
        assert "`grpc`" in self.spec_content

    def test_has_database_condition(self) -> None:
        """仕様書に has_database 条件が記載。"""
        assert "has_database" in self.spec_content

    def test_database_type_condition(self) -> None:
        """仕様書に database_type 条件が記載。"""
        assert "database_type" in self.spec_content
        assert "`postgresql`" in self.spec_content
        assert "`mysql`" in self.spec_content


class TestCicdGenerationExamples:
    """テンプレート仕様-CICD.md: 生成例整合性検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_go_rest_server_example(self) -> None:
        """仕様書に Go REST サーバーの生成例が記載。"""
        assert "order-api" in self.spec_content
        assert "order-api-ci.yaml" in self.spec_content
        assert "order-api-deploy.yaml" in self.spec_content

    def test_rust_grpc_server_example(self) -> None:
        """仕様書に Rust gRPC サーバーの生成例が記載。"""
        assert "auth-service" in self.spec_content
        assert "auth-service-ci.yaml" in self.spec_content

    def test_react_client_example(self) -> None:
        """仕様書に React クライアントの生成例が記載。"""
        assert "ledger-app" in self.spec_content
        assert "ledger-app-ci.yaml" in self.spec_content


class TestCicdTemplateVariableUsage:
    """テンプレート仕様-CICD.md: テンプレート変数使用検証。"""

    def setup_method(self) -> None:
        self.ci_content = (CICD / "ci.yaml.tera").read_text(encoding="utf-8")
        self.deploy_content = (CICD / "deploy.yaml.tera").read_text(encoding="utf-8")

    def test_api_style_variable(self) -> None:
        """CI テンプレートに api_style 変数が使用されている。"""
        assert "api_style" in self.ci_content

    def test_has_database_variable(self) -> None:
        """CI テンプレートに has_database 変数が使用されている。"""
        assert "has_database" in self.ci_content

    def test_framework_variable(self) -> None:
        """CI テンプレートに framework 変数が使用されている。"""
        assert "framework" in self.ci_content

    def test_go_module_or_module_path_variable(self) -> None:
        """CI テンプレートに module_path 変数が使用されている（go_module 相当）。"""
        assert "module_path" in self.ci_content

    def test_rust_crate_or_module_path_variable(self) -> None:
        """CI テンプレートに module_path 変数が使用されている（rust_crate 相当）。"""
        assert "module_path" in self.ci_content

    def test_service_name_snake_in_spec(self) -> None:
        """仕様書に service_name_snake 変数が記載。"""
        spec = SPEC.read_text(encoding="utf-8")
        assert "service_name_snake" in spec


class TestCicdDeploymentPath:
    """テンプレート仕様-CICD.md: 配置パス形式検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_ci_deployment_path(self) -> None:
        """仕様書に CI の配置パスが .github/workflows/{{ service_name }}-ci.yaml と記載。"""
        assert "{{ service_name }}-ci.yaml" in self.spec_content

    def test_deploy_deployment_path(self) -> None:
        """仕様書に Deploy の配置パスが .github/workflows/{{ service_name }}-deploy.yaml と記載。"""
        assert "{{ service_name }}-deploy.yaml" in self.spec_content


class TestCicdBufVersion:
    """テンプレート仕様-CICD.md: buf バージョン 1.47.2 検証。"""

    def setup_method(self) -> None:
        self.ci_content = (CICD / "ci.yaml.tera").read_text(encoding="utf-8")
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_buf_version_in_template(self) -> None:
        """CI テンプレートに buf バージョン 1.47.2 が指定。"""
        assert "1.47.2" in self.ci_content

    def test_buf_version_in_spec(self) -> None:
        """仕様書に buf バージョン 1.47.2 が記載。"""
        assert "1.47.2" in self.spec_content


class TestCicdDartVersion:
    """テンプレート仕様-CICD.md: Dart バージョン 3.5 検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_dart_version_in_spec(self) -> None:
        """仕様書に Dart バージョン 3.5 が記載。"""
        assert "3.5" in self.spec_content
