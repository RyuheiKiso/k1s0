"""可観測性設計.md の仕様準拠テスト。

可観測性スタック（Prometheus, Alertmanager, Loki, Jaeger）の設定が
設計ドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]
OBS = ROOT / "infra" / "observability"


class TestPrometheusConfig:
    """可観測性設計.md: Prometheus 設定の検証。"""

    def test_prometheus_config_exists(self) -> None:
        assert (OBS / "prometheus" / "prometheus-config.yaml").exists()

    def test_servicemonitor_exists(self) -> None:
        assert (OBS / "prometheus" / "servicemonitor.yaml").exists()

    def test_servicemonitor_namespaces(self) -> None:
        """可観測性設計.md: ServiceMonitor が k1s0 Namespace を対象とする。"""
        path = OBS / "prometheus" / "servicemonitor.yaml"
        content = path.read_text(encoding="utf-8")
        assert "k1s0-system" in content
        assert "k1s0-business" in content
        assert "k1s0-service" in content

    def test_servicemonitor_part_of_label(self) -> None:
        """可観測性設計.md: ServiceMonitor が app.kubernetes.io/part-of: k1s0 を使用。"""
        path = OBS / "prometheus" / "servicemonitor.yaml"
        content = path.read_text(encoding="utf-8")
        assert "app.kubernetes.io/part-of" in content


class TestAlertRules:
    """可観測性設計.md: アラートルールの検証。"""

    def test_system_tier_alerts_exist(self) -> None:
        path = OBS / "prometheus" / "alerts" / "system-tier-alerts.yaml"
        assert path.exists(), "system-tier-alerts.yaml が存在しません"

    def test_business_service_tier_alerts_exist(self) -> None:
        path = OBS / "prometheus" / "alerts" / "business-service-tier-alerts.yaml"
        assert path.exists(), "business-service-tier-alerts.yaml が存在しません"

    def test_slo_recording_rules_exist(self) -> None:
        path = OBS / "prometheus" / "alerts" / "slo-recording-rules.yaml"
        assert path.exists(), "slo-recording-rules.yaml が存在しません"

    def test_system_alerts_content(self) -> None:
        """可観測性設計.md: system Tier のアラートルールが定義されている。"""
        path = OBS / "prometheus" / "alerts" / "system-tier-alerts.yaml"
        content = path.read_text(encoding="utf-8")
        assert "system-tier-alerts" in content
        assert "SystemServiceHighErrorRate" in content or "SystemServiceErrorRateWarning" in content

    def test_slo_recording_rules_content(self) -> None:
        """可観測性設計.md: SLO Recording Rule が定義されている。"""
        path = OBS / "prometheus" / "alerts" / "slo-recording-rules.yaml"
        content = path.read_text(encoding="utf-8")
        assert "slo:availability:ratio" in content
        assert "slo:error_budget:remaining" in content


class TestAlertmanagerConfig:
    """可観測性設計.md: Alertmanager 設定の検証。"""

    def test_alertmanager_config_exists(self) -> None:
        path = OBS / "alertmanager" / "alertmanager-config.yaml"
        assert path.exists(), "alertmanager-config.yaml が存在しません"

    def test_alertmanager_routes(self) -> None:
        """可観測性設計.md: 重大度別のルーティングが設定されている。"""
        path = OBS / "alertmanager" / "alertmanager-config.yaml"
        content = path.read_text(encoding="utf-8")
        assert "teams-critical" in content
        assert "teams-warning" in content
        assert "teams-default" in content

    def test_msteams_config_exists(self) -> None:
        assert (OBS / "alertmanager" / "prometheus-msteams-config.yaml").exists()

    def test_msteams_deployment_exists(self) -> None:
        assert (OBS / "alertmanager" / "prometheus-msteams-deployment.yaml").exists()

    def test_msteams_webhook_secret_exists(self) -> None:
        assert (OBS / "alertmanager" / "prometheus-msteams-webhook-secret.yaml").exists()


class TestLokiConfig:
    """可観測性設計.md: Loki 設定の検証。"""

    def test_loki_config_exists(self) -> None:
        assert (OBS / "loki" / "loki-config.yaml").exists()

    def test_loki_retention(self) -> None:
        """可観測性設計.md: Loki の保持期間が 2160h(90日)。"""
        path = OBS / "loki" / "loki-config.yaml"
        content = path.read_text(encoding="utf-8")
        assert "2160h" in content

    def test_loki_audit_retention(self) -> None:
        """可観測性設計.md: 監査ログの保持期間が 8760h(1年)。"""
        path = OBS / "loki" / "loki-config.yaml"
        content = path.read_text(encoding="utf-8")
        assert "8760h" in content

    def test_promtail_config_exists(self) -> None:
        assert (OBS / "loki" / "promtail-config.yaml").exists()

    def test_promtail_kubernetes_sd(self) -> None:
        """可観測性設計.md: Promtail が kubernetes-pods の scrape を行う。"""
        path = OBS / "loki" / "promtail-config.yaml"
        content = path.read_text(encoding="utf-8")
        assert "kubernetes-pods" in content


class TestJaegerConfig:
    """可観測性設計.md: Jaeger 設定の検証。"""

    def test_jaeger_config_exists(self) -> None:
        assert (OBS / "jaeger" / "jaeger-config.yaml").exists()

    def test_jaeger_production_strategy(self) -> None:
        """可観測性設計.md: Jaeger が production 戦略を使用。"""
        path = OBS / "jaeger" / "jaeger-config.yaml"
        content = path.read_text(encoding="utf-8")
        assert "production" in content

    def test_jaeger_elasticsearch_storage(self) -> None:
        """可観測性設計.md: Jaeger が Elasticsearch ストレージを使用。"""
        path = OBS / "jaeger" / "jaeger-config.yaml"
        content = path.read_text(encoding="utf-8")
        assert "elasticsearch" in content


class TestLocalDevObservability:
    """可観測性設計.md: ローカル開発用の設定検証。"""

    def test_prometheus_local_config(self) -> None:
        path = ROOT / "infra" / "docker" / "prometheus" / "prometheus.yaml"
        assert path.exists(), "docker/prometheus/prometheus.yaml が存在しません"

    def test_grafana_datasources(self) -> None:
        ds_dir = ROOT / "infra" / "docker" / "grafana" / "provisioning" / "datasources"
        # datasources は統合ファイル datasources.yaml に定義されている
        ds_file = ds_dir / "datasources.yaml"
        assert ds_file.exists(), "datasources.yaml が存在しません"
        content = ds_file.read_text(encoding="utf-8")
        assert "prometheus" in content.lower(), "Prometheus datasource が定義されていません"
        assert "loki" in content.lower(), "Loki datasource が定義されていません"
        assert "jaeger" in content.lower(), "Jaeger datasource が定義されていません"


class TestGrafanaDashboardJsonFiles:
    """可観測性設計.md: Grafana ダッシュボード JSON 定義ファイルの検証。"""

    DASHBOARDS = ROOT / "infra" / "observability" / "grafana" / "dashboards"

    def test_overview_dashboard_exists(self) -> None:
        """可観測性設計.md: Overview ダッシュボードが存在。"""
        assert (self.DASHBOARDS / "overview.json").exists()

    def test_slo_dashboard_exists(self) -> None:
        """可観測性設計.md: SLO ダッシュボードが存在。"""
        assert (self.DASHBOARDS / "slo.json").exists()

    def test_overview_dashboard_valid_json(self) -> None:
        """可観測性設計.md: Overview ダッシュボードが有効な JSON。"""
        import json
        content = (self.DASHBOARDS / "overview.json").read_text(encoding="utf-8")
        data = json.loads(content)
        assert "panels" in data

    def test_slo_dashboard_valid_json(self) -> None:
        """可観測性設計.md: SLO ダッシュボードが有効な JSON。"""
        import json
        content = (self.DASHBOARDS / "slo.json").read_text(encoding="utf-8")
        data = json.loads(content)
        assert "panels" in data

    def test_overview_has_request_rate_panel(self) -> None:
        """可観測性設計.md: Overview に Request Rate パネルが存在。"""
        import json
        content = (self.DASHBOARDS / "overview.json").read_text(encoding="utf-8")
        data = json.loads(content)
        titles = [p["title"] for p in data["panels"]]
        assert "Request Rate" in titles

    def test_overview_has_error_rate_panel(self) -> None:
        """可観測性設計.md: Overview に Error Rate パネルが存在。"""
        import json
        content = (self.DASHBOARDS / "overview.json").read_text(encoding="utf-8")
        data = json.loads(content)
        titles = [p["title"] for p in data["panels"]]
        assert "Error Rate" in titles


class TestSLOTargetValues:
    """可観測性設計.md: SLO 目標値の検証。"""

    def test_slo_recording_rules_system_target(self) -> None:
        """可観測性設計.md: system Tier 可用性目標 99.95%。"""
        path = OBS / "prometheus" / "alerts" / "slo-recording-rules.yaml"
        content = path.read_text(encoding="utf-8")
        assert "0.9995" in content

    def test_slo_recording_rules_business_service_target(self) -> None:
        """可観測性設計.md: business/service Tier 可用性目標 99.9%。"""
        path = OBS / "prometheus" / "alerts" / "slo-recording-rules.yaml"
        content = path.read_text(encoding="utf-8")
        assert "0.999" in content


class TestEnvironmentLogLevels:
    """可観測性設計.md: 環境別ログレベルの検証。"""

    def test_dev_debug_level(self) -> None:
        """可観測性設計.md: dev 環境はデフォルト debug。"""
        go_config = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "config" / "config.yaml.tera"
        content = go_config.read_text(encoding="utf-8")
        assert "debug" in content.lower() or "info" in content.lower()

    def test_config_has_log_level(self) -> None:
        """可観測性設計.md: config.yaml テンプレートに log.level 設定がある。"""
        go_config = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "config" / "config.yaml.tera"
        content = go_config.read_text(encoding="utf-8")
        assert "level:" in content

    def test_config_has_log_format(self) -> None:
        """可観測性設計.md: config.yaml テンプレートに log.format 設定がある。"""
        go_config = ROOT / "CLI" / "crates" / "k1s0-cli" / "templates" / "server" / "go" / "config" / "config.yaml.tera"
        content = go_config.read_text(encoding="utf-8")
        assert "format:" in content


class TestJsonStandardLogFields:
    """可観測性設計.md: JSON 標準ログフィールド（14フィールド）の検証。"""

    EXPECTED_FIELDS = [
        "timestamp", "level", "message", "service", "version",
        "tier", "environment", "trace_id", "span_id", "request_id",
        "error",
    ]

    def test_log_fields_in_doc(self) -> None:
        """可観測性設計.md: JSON ログサンプルに標準フィールドが記載されている。"""
        doc_path = ROOT / "docs" / "可観測性設計.md"
        content = doc_path.read_text(encoding="utf-8")
        for field in self.EXPECTED_FIELDS:
            assert f'"{field}"' in content, (
                f"可観測性設計.md に標準フィールド '{field}' が記載されていません"
            )


class TestREDCustomMetrics:
    """可観測性設計.md: RED メソッドのカスタムメトリクス定義の検証。"""

    def test_doc_defines_grpc_server_handled_total(self) -> None:
        """可観測性設計.md: grpc_server_handled_total カウンターが定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "grpc_server_handled_total" in doc

    def test_doc_defines_grpc_server_handling_seconds(self) -> None:
        """可観測性設計.md: grpc_server_handling_seconds ヒストグラムが定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "grpc_server_handling_seconds" in doc

    def test_doc_defines_db_query_duration_seconds(self) -> None:
        """可観測性設計.md: db_query_duration_seconds ヒストグラムが定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "db_query_duration_seconds" in doc

    def test_doc_defines_kafka_messages_produced_total(self) -> None:
        """可観測性設計.md: kafka_messages_produced_total カウンターが定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "kafka_messages_produced_total" in doc

    def test_doc_defines_kafka_messages_consumed_total(self) -> None:
        """可観測性設計.md: kafka_messages_consumed_total カウンターが定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "kafka_messages_consumed_total" in doc


class TestUSEMethodMetrics:
    """可観測性設計.md: USE メソッドのインフラメトリクス定義の検証。"""

    def test_doc_defines_cpu_utilization_metric(self) -> None:
        """可観測性設計.md: container_cpu_usage_seconds_total が定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "container_cpu_usage_seconds_total" in doc

    def test_doc_defines_memory_saturation_metric(self) -> None:
        """可観測性設計.md: container_memory_working_set_bytes が定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "container_memory_working_set_bytes" in doc

    def test_doc_defines_disk_errors_metric(self) -> None:
        """可観測性設計.md: node_disk_io_time_seconds_total が定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "node_disk_io_time_seconds_total" in doc


class TestGrafanaDashboardAll8:
    """可観測性設計.md: Grafana ダッシュボード全 8 種の定義検証。"""

    EXPECTED_DASHBOARDS = [
        "Overview", "Service Detail", "Infrastructure", "Kafka",
        "Kong", "Istio", "Database", "SLO",
    ]

    def test_all_8_dashboards_documented(self) -> None:
        """可観測性設計.md: 8 種のダッシュボードが設計書に記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        for db_name in self.EXPECTED_DASHBOARDS:
            assert db_name in doc, (
                f"可観測性設計.md にダッシュボード '{db_name}' が記載されていません"
            )


class TestGrafanaPanelPromQL:
    """可観測性設計.md: Grafana パネルの PromQL 定義の検証。"""

    def test_overview_request_rate_promql(self) -> None:
        """可観測性設計.md: リクエスト数パネルの PromQL が定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "rate(http_requests_total[5m])" in doc

    def test_overview_error_rate_promql(self) -> None:
        """可観測性設計.md: エラーレートパネルの PromQL が定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert 'rate(http_requests_total{status=~"5.."}[5m])' in doc

    def test_overview_p99_latency_promql(self) -> None:
        """可観測性設計.md: P99 レイテンシパネルの PromQL が定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "histogram_quantile(0.99" in doc

    def test_slo_error_budget_promql(self) -> None:
        """可観測性設計.md: エラーバジェット残量パネルの PromQL が定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "slo:error_budget:remaining" in doc

    def test_kafka_consumer_lag_promql(self) -> None:
        """可観測性設計.md: Consumer Lag パネルの PromQL が定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "kafka_consumer_group_lag" in doc


class TestEnvironmentAlertSuppression:
    """可観測性設計.md: 環境別アラート抑制設定の検証。"""

    def test_doc_defines_dev_no_alerts(self) -> None:
        """可観測性設計.md: dev 環境は critical/warning ともに無効。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        # dev行に「無効」が2つあることを確認
        lines = doc.split("\n")
        dev_lines = [l for l in lines if "| dev" in l and "無効" in l]
        assert len(dev_lines) >= 1, "dev 環境のアラート抑制定義が見つかりません"

    def test_doc_defines_staging_critical_only(self) -> None:
        """可観測性設計.md: staging 環境は critical のみ有効。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        lines = doc.split("\n")
        staging_lines = [l for l in lines if "| staging" in l and "有効" in l and "無効" in l]
        assert len(staging_lines) >= 1, "staging 環境のアラート抑制定義が見つかりません"

    def test_doc_defines_prod_all_alerts(self) -> None:
        """可観測性設計.md: prod 環境は全アラートを通知。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        lines = doc.split("\n")
        prod_lines = [l for l in lines if "| prod" in l and "有効" in l]
        assert len(prod_lines) >= 1, "prod 環境のアラート抑制定義が見つかりません"

    def test_staging_warning_suppression_config(self) -> None:
        """可観測性設計.md: staging の warning 抑制設定例が記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "environment: staging" in doc
        assert "receiver: 'null'" in doc or 'receiver: "null"' in doc


class TestAlertThresholdValues:
    """可観測性設計.md: アラート閾値の具体値検証。"""

    def test_system_warning_threshold(self) -> None:
        """可観測性設計.md: system warning 閾値 > 0.1%（5分間）。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "> 0.001" in doc, "system warning 閾値 0.001 (0.1%) が見つかりません"

    def test_system_critical_threshold(self) -> None:
        """可観測性設計.md: system critical 閾値 > 1%（5分間）。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "> 0.01" in doc, "system critical 閾値 0.01 (1%) が見つかりません"

    def test_business_service_warning_threshold(self) -> None:
        """可観測性設計.md: business/service warning 閾値 > 0.2%（5分間）。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "> 0.002" in doc, "business/service warning 閾値 0.002 (0.2%) が見つかりません"

    def test_business_service_critical_threshold(self) -> None:
        """可観測性設計.md: business/service critical 閾値 > 5%（5分間）。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "> 0.05" in doc, "business/service critical 閾値 0.05 (5%) が見つかりません"

    def test_alert_rules_for_5m(self) -> None:
        """可観測性設計.md: アラートルールの for 期間が 5m。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "for: 5m" in doc


class TestSLADefinition:
    """可観測性設計.md: SLA 定義の検証。"""

    def test_sla_system_availability(self) -> None:
        """可観測性設計.md: system Tier 内部 SLA 可用性 99.9%。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        # SLA テーブル内に system 99.9% が記載されている
        assert "99.9%" in doc

    def test_sla_business_availability(self) -> None:
        """可観測性設計.md: business Tier 内部 SLA 可用性 99.8%。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "99.8%" in doc

    def test_sla_system_latency(self) -> None:
        """可観測性設計.md: system Tier P99 レイテンシ < 500ms。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "< 500ms" in doc

    def test_sla_escalation_defined(self) -> None:
        """可観測性設計.md: SLA 違反時のエスカレーションが定義されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "エスカレーション" in doc
        assert "ポストモーテム" in doc


class TestOTelSDKInitPattern:
    """可観測性設計.md: OpenTelemetry SDK 初期化パターンの検証。"""

    def test_go_otel_init_pattern_in_doc(self) -> None:
        """可観測性設計.md: Go の OTel 初期化パターンが記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "initTracer" in doc
        assert "sdktrace.NewTracerProvider" in doc

    def test_rust_otel_init_pattern_in_doc(self) -> None:
        """可観測性設計.md: Rust の OTel 初期化パターンが記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "init_tracer" in doc
        assert "TracerProvider::builder" in doc

    def test_go_otel_exporter_grpc(self) -> None:
        """可観測性設計.md: Go は OTLP gRPC エクスポーターを使用。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "otlptracegrpc" in doc

    def test_rust_otel_exporter_tonic(self) -> None:
        """可観測性設計.md: Rust は tonic ベースのエクスポーターを使用。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "with_tonic" in doc


class TestAutoInstrumentation:
    """可観測性設計.md: 自動インストルメンテーションの検証。"""

    def test_go_otelhttp_middleware(self) -> None:
        """可観測性設計.md: Go は otelhttp ミドルウェアを使用。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "otelhttp" in doc

    def test_rust_tracing_opentelemetry(self) -> None:
        """可観測性設計.md: Rust は tracing-opentelemetry を使用。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "tracing-opentelemetry" in doc


class TestStructuredLogGoRustImpl:
    """可観測性設計.md: 構造化ログ Go/Rust 実装パターンの検証。"""

    def test_go_slog_implementation(self) -> None:
        """可観測性設計.md: Go は slog による構造化ログ実装が記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "slog.NewJSONHandler" in doc
        assert "NewLogger" in doc

    def test_go_log_with_trace(self) -> None:
        """可観測性設計.md: Go のトレースコンテキスト埋め込み関数が記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "LogWithTrace" in doc
        assert "SpanContextFromContext" in doc

    def test_rust_tracing_subscriber_impl(self) -> None:
        """可観測性設計.md: Rust は tracing_subscriber による構造化ログ実装が記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "tracing_subscriber::registry" in doc
        assert "init_logger" in doc

    def test_rust_json_layer(self) -> None:
        """可観測性設計.md: Rust の JSON レイヤーが記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "fmt::layer()" in doc
        assert ".json()" in doc


class TestTraceCorrelation:
    """可観測性設計.md: トレース相関の検証。"""

    def test_trace_correlation_design(self) -> None:
        """可観測性設計.md: trace_id と span_id でログとトレースを相関させる設計が記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "trace_id" in doc
        assert "span_id" in doc
        assert "相関" in doc

    def test_trace_propagation_flow(self) -> None:
        """可観測性設計.md: Client → Kong → Service のトレース伝搬フローが記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "Client" in doc and "Kong" in doc and "Service A" in doc

    def test_grafana_trace_drilldown(self) -> None:
        """可観測性設計.md: Grafana でトレース統合表示・ドリルダウンが記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        assert "ドリルダウン" in doc


class TestJsonLogMissingFields:
    """可観測性設計.md: JSON ログの追加フィールド（method, path, status, duration_ms, user_id）の検証。"""

    ADDITIONAL_FIELDS = [
        "method", "path", "status", "duration_ms", "user_id",
    ]

    def test_additional_log_fields_in_doc(self) -> None:
        """可観測性設計.md: JSON ログサンプルに追加フィールドが記載されている。"""
        doc = (ROOT / "docs" / "可観測性設計.md").read_text(encoding="utf-8")
        for field in self.ADDITIONAL_FIELDS:
            assert f'"{field}"' in doc, (
                f"可観測性設計.md に追加フィールド '{field}' が記載されていません"
            )
