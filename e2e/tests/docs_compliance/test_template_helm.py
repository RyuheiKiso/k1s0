"""テンプレート仕様-Helm.md の仕様準拠テスト。

Helmテンプレート仕様書とテンプレートファイルの検証。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TEMPLATES = ROOT / "CLI" / "templates"
DOCS = ROOT / "docs"
SPEC = DOCS / "テンプレート仕様-Helm.md"
HELM = TEMPLATES / "helm"


class TestHelmSpecExists:
    """仕様書ファイルの存在確認。"""

    def test_spec_file_exists(self) -> None:
        assert SPEC.exists(), "テンプレート仕様-Helm.md が存在しません"


class TestHelmTemplateFilesExist:
    """テンプレートファイルの存在確認。"""

    @pytest.mark.parametrize(
        "template",
        [
            "Chart.yaml.tera",
            "values.yaml.tera",
            "values-dev.yaml.tera",
            "values-staging.yaml.tera",
            "values-prod.yaml.tera",
            "templates/deployment.yaml.tera",
            "templates/service.yaml.tera",
            "templates/configmap.yaml.tera",
            "templates/hpa.yaml.tera",
            "templates/pdb.yaml.tera",
        ],
    )
    def test_helm_template_exists(self, template: str) -> None:
        path = HELM / template
        assert path.exists(), f"helm/{template} が存在しません"


class TestHelmSpecSections:
    """仕様書の主要セクション存在チェック。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "section",
        [
            "## 概要",
            "## 配置パス",
            "## テンプレートファイル一覧",
            "## 条件付き生成",
            "## Tera / Helm 構文衝突の回避",
            "## テンプレート変数マッピング",
            "## テンプレート詳細",
            "## テンプレート変数一覧",
        ],
    )
    def test_section_exists(self, section: str) -> None:
        assert section in self.content, f"セクション '{section}' が仕様書に存在しません"


class TestHelmSpecRawSyntax:
    """Tera/Helm構文衝突の回避（{% raw %} の記載）検証。"""

    def setup_method(self) -> None:
        self.content = SPEC.read_text(encoding="utf-8")

    def test_raw_block_documented(self) -> None:
        assert "{% raw %}" in self.content, "{% raw %} が仕様書に記載されていません"

    def test_endraw_block_documented(self) -> None:
        assert "{% endraw %}" in self.content, "{% endraw %} が仕様書に記載されていません"


class TestHelmTemplateVariables:
    """テンプレート変数の使用チェック。"""

    @pytest.mark.parametrize(
        "template,variable",
        [
            ("Chart.yaml.tera", "{{ service_name }}"),
            ("Chart.yaml.tera", "{{ service_name_pascal }}"),
            ("values.yaml.tera", "{{ docker_registry }}"),
            ("values.yaml.tera", "{{ docker_project }}"),
            ("values.yaml.tera", "{{ service_name }}"),
            ("values.yaml.tera", "{{ tier }}"),
            ("values-dev.yaml.tera", "has_database"),
            ("values-staging.yaml.tera", "has_database"),
            ("values-prod.yaml.tera", "{{ service_name }}"),
        ],
    )
    def test_template_variable_used(self, template: str, variable: str) -> None:
        path = HELM / template
        content = path.read_text(encoding="utf-8")
        assert variable in content, f"helm/{template} に変数 '{variable}' が含まれていません"

    @pytest.mark.parametrize(
        "template,include_call",
        [
            ("templates/deployment.yaml.tera", "k1s0-common.deployment"),
            ("templates/service.yaml.tera", "k1s0-common.service"),
            ("templates/configmap.yaml.tera", "k1s0-common.configmap"),
            ("templates/hpa.yaml.tera", "k1s0-common.hpa"),
            ("templates/pdb.yaml.tera", "k1s0-common.pdb"),
        ],
    )
    def test_library_chart_include(self, template: str, include_call: str) -> None:
        path = HELM / template
        content = path.read_text(encoding="utf-8")
        assert include_call in content, (
            f"helm/{template} に Library Chart 呼び出し '{include_call}' が含まれていません"
        )


class TestHelmValuesStagingContent:
    """テンプレート仕様-Helm.md: values-staging.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values-staging.yaml.tera").read_text(encoding="utf-8")

    def test_replica_count(self) -> None:
        assert "replicaCount: 2" in self.content

    def test_autoscaling_enabled(self) -> None:
        assert "enabled: true" in self.content

    def test_min_replicas(self) -> None:
        assert "minReplicas: 2" in self.content

    def test_environment_staging(self) -> None:
        assert "environment: staging" in self.content

    def test_database_conditional(self) -> None:
        assert "has_database" in self.content

    def test_ssl_mode_require(self) -> None:
        """テンプレート仕様-Helm.md: ステージングはssl_mode: require。"""
        assert "ssl_mode: require" in self.content

    def test_trace_sample_rate(self) -> None:
        assert "sample_rate: 0.5" in self.content


class TestHelmValuesContent:
    """テンプレート仕様-Helm.md: values.yaml.tera の主要フィールド検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_image_section(self) -> None:
        assert "image:" in self.content
        assert "{{ docker_registry }}" in self.content
        assert "{{ docker_project }}" in self.content

    def test_replica_count(self) -> None:
        assert "replicaCount:" in self.content

    def test_resources(self) -> None:
        assert "resources:" in self.content
        assert "requests:" in self.content
        assert "limits:" in self.content

    def test_probes(self) -> None:
        assert "/healthz" in self.content
        assert "/readyz" in self.content

    def test_security_context(self) -> None:
        assert "runAsNonRoot: true" in self.content
        assert "readOnlyRootFilesystem: true" in self.content

    def test_vault_section(self) -> None:
        assert "vault:" in self.content
        assert "{{ tier }}" in self.content

    def test_grpc_port_conditional(self) -> None:
        assert "grpcPort:" in self.content

    def test_kafka_section(self) -> None:
        assert "kafka:" in self.content
        assert "{{ has_kafka }}" in self.content

    def test_redis_section(self) -> None:
        assert "redis:" in self.content
        assert "{{ has_redis }}" in self.content


class TestHelmChartYamlContent:
    """テンプレート仕様-Helm.md: Chart.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "Chart.yaml.tera").read_text(encoding="utf-8")

    def test_api_version(self) -> None:
        assert "apiVersion: v2" in self.content

    def test_service_name(self) -> None:
        assert "{{ service_name }}" in self.content

    def test_description(self) -> None:
        assert "{{ service_name_pascal }}" in self.content

    def test_type_application(self) -> None:
        assert "type: application" in self.content

    def test_k1s0_common_dependency(self) -> None:
        assert "k1s0-common" in self.content

    def test_business_tier_path(self) -> None:
        """テンプレート仕様-Helm.md: business tier のLibrary Chart相対パス。"""
        assert "../../../../charts/k1s0-common" in self.content

    def test_default_tier_path(self) -> None:
        """テンプレート仕様-Helm.md: system/service tier のLibrary Chart相対パス。"""
        assert "../../../charts/k1s0-common" in self.content


class TestHelmTemplateConfigmapContent:
    """テンプレート仕様-Helm.md: configmap.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "templates" / "configmap.yaml.tera").read_text(encoding="utf-8")

    def test_library_chart_call(self) -> None:
        assert "k1s0-common.configmap" in self.content

    def test_raw_endraw(self) -> None:
        """テンプレート仕様-Helm.md: raw/endraw でHelm構文を保護。"""
        assert "{% raw %}" in self.content
        assert "{% endraw %}" in self.content


class TestHelmTemplateHpaContent:
    """テンプレート仕様-Helm.md: hpa.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "templates" / "hpa.yaml.tera").read_text(encoding="utf-8")

    def test_library_chart_call(self) -> None:
        assert "k1s0-common.hpa" in self.content

    def test_raw_endraw(self) -> None:
        assert "{% raw %}" in self.content
        assert "{% endraw %}" in self.content


class TestHelmTemplatePdbContent:
    """テンプレート仕様-Helm.md: pdb.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "templates" / "pdb.yaml.tera").read_text(encoding="utf-8")

    def test_library_chart_call(self) -> None:
        assert "k1s0-common.pdb" in self.content

    def test_raw_endraw(self) -> None:
        assert "{% raw %}" in self.content
        assert "{% endraw %}" in self.content


class TestHelmValuesProdContent:
    """テンプレート仕様-Helm.md: values-prod.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values-prod.yaml.tera").read_text(encoding="utf-8")

    def test_replica_count(self) -> None:
        assert "replicaCount: 3" in self.content

    def test_autoscaling_max_replicas(self) -> None:
        assert "maxReplicas: 10" in self.content

    def test_pod_anti_affinity(self) -> None:
        assert "podAntiAffinity" in self.content

    def test_raw_endraw_syntax(self) -> None:
        """テンプレート仕様-Helm.md: raw/endraw で Go テンプレート構文を保護。"""
        assert "{% raw %}" in self.content
        assert "{% endraw %}" in self.content

    def test_environment_prod(self) -> None:
        assert "environment: prod" in self.content

    def test_ssl_mode_verify_full(self) -> None:
        """テンプレート仕様-Helm.md: 本番はssl_mode: verify-full。"""
        assert "ssl_mode: verify-full" in self.content


class TestHelmValuesDevContent:
    """テンプレート仕様-Helm.md: values-dev.yaml.tera の内容検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values-dev.yaml.tera").read_text(encoding="utf-8")

    def test_replica_count(self) -> None:
        assert "replicaCount: 1" in self.content

    def test_autoscaling_disabled(self) -> None:
        assert "enabled: false" in self.content

    def test_pdb_disabled(self) -> None:
        assert "pdb:" in self.content

    def test_vault_disabled(self) -> None:
        assert "vault:" in self.content

    def test_environment_dev(self) -> None:
        assert "environment: dev" in self.content

    def test_log_level_debug(self) -> None:
        assert "level: debug" in self.content
