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
        assert (ds_dir / "prometheus.yaml").exists()
        assert (ds_dir / "loki.yaml").exists()
        assert (ds_dir / "jaeger.yaml").exists()
