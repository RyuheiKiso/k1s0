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


class TestHelmSpecKindConstraint:
    """テンプレート仕様-Helm.md: kind=server のみ制約検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_server_only_documented(self) -> None:
        """仕様書に kind = server のみ生成と記載。"""
        assert "kind = server のみ" in self.spec_content

    def test_client_not_generated(self) -> None:
        """仕様書に client では Helm Chart を生成しないと記載。"""
        assert "client" in self.spec_content
        assert "library" in self.spec_content
        assert "database" in self.spec_content


class TestHelmSpecTierPaths:
    """テンプレート仕様-Helm.md: 配置パス Tier 別検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_system_path(self) -> None:
        """仕様書に system Tier の配置パスが記載。"""
        assert "infra/helm/services/system/{service_name}/" in self.spec_content

    def test_business_path(self) -> None:
        """仕様書に business Tier の配置パスが記載。"""
        assert "infra/helm/services/business/{domain}/{service_name}/" in self.spec_content

    def test_service_path(self) -> None:
        """仕様書に service Tier の配置パスが記載。"""
        assert "infra/helm/services/service/{service_name}/" in self.spec_content


class TestHelmGrpcPortValue:
    """テンプレート仕様-Helm.md: grpcPort 50051 値検証。"""

    def setup_method(self) -> None:
        self.values_content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_grpc_port_50051(self) -> None:
        """values.yaml.tera に grpcPort: 50051 が含まれる。"""
        assert "grpcPort: 50051" in self.values_content


class TestHelmKafkaConditional:
    """テンプレート仕様-Helm.md: has_kafka 条件分岐値検証。"""

    def setup_method(self) -> None:
        self.values_content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_kafka_enabled_variable(self) -> None:
        """values.yaml.tera に kafka.enabled: {{ has_kafka }} が含まれる。"""
        assert "{{ has_kafka }}" in self.values_content

    def test_kafka_brokers(self) -> None:
        """values.yaml.tera に kafka.brokers が含まれる。"""
        assert "brokers:" in self.values_content


class TestHelmRedisConditional:
    """テンプレート仕様-Helm.md: has_redis 条件分岐値検証。"""

    def setup_method(self) -> None:
        self.values_content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_redis_enabled_variable(self) -> None:
        """values.yaml.tera に redis.enabled: {{ has_redis }} が含まれる。"""
        assert "{{ has_redis }}" in self.values_content

    def test_redis_host(self) -> None:
        """values.yaml.tera に redis.host が含まれる。"""
        assert "host:" in self.values_content


class TestHelmImagePullSecrets:
    """テンプレート仕様-Helm.md: imagePullSecrets 検証。"""

    def setup_method(self) -> None:
        self.values_content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_image_pull_secrets(self) -> None:
        """values.yaml.tera に imagePullSecrets が含まれる。"""
        assert "imagePullSecrets:" in self.values_content
        assert "harbor-pull-secret" in self.values_content


class TestHelmReplicaCountDefault:
    """テンプレート仕様-Helm.md: replicaCount デフォルト 2 検証。"""

    def setup_method(self) -> None:
        self.values_content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_replica_count_default_2(self) -> None:
        """values.yaml.tera のデフォルト replicaCount が 2。"""
        assert "replicaCount: 2" in self.values_content


class TestHelmValuesYamlDetails:
    """テンプレート仕様-Helm.md: values.yaml 具体値検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_resources_requests_cpu(self) -> None:
        """values.yaml.tera に cpu: 250m のリクエストがある。"""
        assert "cpu: 250m" in self.content

    def test_resources_requests_memory(self) -> None:
        """values.yaml.tera に memory: 256Mi のリクエストがある。"""
        assert "memory: 256Mi" in self.content

    def test_resources_limits_cpu(self) -> None:
        """values.yaml.tera に cpu: 1000m のリミットがある。"""
        assert "cpu: 1000m" in self.content

    def test_resources_limits_memory(self) -> None:
        """values.yaml.tera に memory: 1Gi のリミットがある。"""
        assert "memory: 1Gi" in self.content

    def test_probes_liveness_healthz(self) -> None:
        """values.yaml.tera に /healthz のプローブがある。"""
        assert "/healthz" in self.content
        assert "initialDelaySeconds: 10" in self.content

    def test_probes_readiness_readyz(self) -> None:
        """values.yaml.tera に /readyz のプローブがある。"""
        assert "/readyz" in self.content
        assert "initialDelaySeconds: 5" in self.content

    def test_autoscaling_defaults(self) -> None:
        """values.yaml.tera に autoscaling のデフォルト値がある。"""
        assert "enabled: true" in self.content
        assert "minReplicas: 2" in self.content
        assert "maxReplicas: 5" in self.content
        assert "targetCPUUtilizationPercentage: 70" in self.content

    def test_pdb_defaults(self) -> None:
        """values.yaml.tera に pdb のデフォルト値がある。"""
        assert "minAvailable: 1" in self.content

    def test_security_context(self) -> None:
        """values.yaml.tera に securityContext がある。"""
        assert "runAsNonRoot: true" in self.content
        assert "readOnlyRootFilesystem: true" in self.content
        assert "allowPrivilegeEscalation: false" in self.content
        assert 'drop: ["ALL"]' in self.content

    def test_vault_secrets(self) -> None:
        """values.yaml.tera に vault secrets セクションがある。"""
        assert "vault:" in self.content
        assert "secret/data/k1s0" in self.content


class TestHelmValuesYamlServiceAccount:
    """テンプレート仕様-Helm.md: values.yaml serviceAccount 検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_service_account_create(self) -> None:
        """values.yaml.tera に serviceAccount.create: true がある。"""
        assert "serviceAccount:" in self.content
        assert "create: true" in self.content


class TestHelmValuesYamlConfigMountPath:
    """テンプレート仕様-Helm.md: values.yaml config.mountPath 検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values.yaml.tera").read_text(encoding="utf-8")

    def test_config_mount_path(self) -> None:
        """values.yaml.tera に config.mountPath: /etc/app がある。"""
        assert "mountPath: /etc/app" in self.content


class TestHelmValuesDevDetails:
    """テンプレート仕様-Helm.md: values-dev 具体値検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values-dev.yaml.tera").read_text(encoding="utf-8")

    def test_resources_requests_cpu(self) -> None:
        assert "cpu: 100m" in self.content

    def test_resources_requests_memory(self) -> None:
        assert "memory: 128Mi" in self.content

    def test_database_conditional(self) -> None:
        """values-dev に has_database 条件がある。"""
        assert "has_database" in self.content

    def test_observability_trace_sample_rate(self) -> None:
        assert "sample_rate: 1.0" in self.content


class TestHelmValuesStagingDetails:
    """テンプレート仕様-Helm.md: values-staging 具体値検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values-staging.yaml.tera").read_text(encoding="utf-8")

    def test_resources_requests_cpu(self) -> None:
        assert "cpu: 250m" in self.content

    def test_resources_requests_memory(self) -> None:
        assert "memory: 256Mi" in self.content

    def test_max_replicas(self) -> None:
        assert "maxReplicas: 5" in self.content


class TestHelmValuesProdDetails:
    """テンプレート仕様-Helm.md: values-prod 具体値検証。"""

    def setup_method(self) -> None:
        self.content = (HELM / "values-prod.yaml.tera").read_text(encoding="utf-8")

    def test_resources_requests_cpu(self) -> None:
        assert "cpu: 500m" in self.content

    def test_resources_requests_memory(self) -> None:
        assert "memory: 512Mi" in self.content

    def test_min_replicas(self) -> None:
        assert "minReplicas: 3" in self.content

    def test_database_conditional(self) -> None:
        """values-prod に has_database 条件がある。"""
        assert "has_database" in self.content

    def test_observability_log_level_warn(self) -> None:
        assert "level: warn" in self.content

    def test_observability_trace_sample_rate(self) -> None:
        assert "sample_rate: 0.1" in self.content


class TestHelmDeploymentServiceRawEndraw:
    """テンプレート仕様-Helm.md: deployment.yaml/service.yaml raw/endraw 検証。"""

    @pytest.mark.parametrize(
        "template",
        [
            "templates/deployment.yaml.tera",
            "templates/service.yaml.tera",
        ],
    )
    def test_template_has_raw_endraw(self, template: str) -> None:
        """Helm テンプレートに raw/endraw がある。"""
        content = (HELM / template).read_text(encoding="utf-8")
        assert "{% raw %}" in content
        assert "{% endraw %}" in content


class TestHelmCommonChartRelativePath:
    """テンプレート仕様-Helm.md: common_chart_relative_path 変数検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")
        self.chart_content = (HELM / "Chart.yaml.tera").read_text(encoding="utf-8")

    def test_relative_path_documented(self) -> None:
        """仕様書に Library Chart 相対パスの導出が記載。"""
        assert "../../charts/k1s0-common" in self.spec_content

    def test_business_four_levels(self) -> None:
        """Chart.yaml.tera に business Tier 用の 4 階層パスがある。"""
        assert "../../../../charts/k1s0-common" in self.chart_content

    def test_default_three_levels(self) -> None:
        """Chart.yaml.tera に system/service Tier 用の 3 階層パスがある。"""
        assert "../../../charts/k1s0-common" in self.chart_content


class TestHelmDomainVariable:
    """テンプレート仕様-Helm.md: domain 変数検証。"""

    def setup_method(self) -> None:
        self.spec_content = SPEC.read_text(encoding="utf-8")

    def test_domain_variable_documented(self) -> None:
        """仕様書に domain 変数が記載。"""
        assert "domain" in self.spec_content

    def test_domain_used_in_business_path(self) -> None:
        """仕様書の business Tier パスに domain が使われている。"""
        assert "{domain}" in self.spec_content
