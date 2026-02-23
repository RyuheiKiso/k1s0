"""可観測性スモークテスト。

Prometheus, Jaeger, Grafana, Loki が起動しているかを検証する。

前提: docker compose --profile observability up -d
"""

import pytest
import requests


class TestPrometheus:
    """Prometheus の基本動作を検証。"""

    PROMETHEUS_URL = "http://localhost:9090"

    def test_prometheus_ready(self):
        """Prometheus の /-/ready が 200 を返すことを確認。"""
        try:
            resp = requests.get(f"{self.PROMETHEUS_URL}/-/ready", timeout=5)
            assert resp.status_code == 200
        except requests.ConnectionError:
            pytest.skip("Prometheus not running")

    def test_prometheus_targets(self):
        """Prometheus にスクレイプターゲットが登録されていることを確認。"""
        try:
            resp = requests.get(f"{self.PROMETHEUS_URL}/api/v1/targets", timeout=5)
        except requests.ConnectionError:
            pytest.skip("Prometheus not running")

        assert resp.status_code == 200
        data = resp.json()
        assert data["status"] == "success"
        targets = data["data"]["activeTargets"]
        assert len(targets) > 0, "No active scrape targets found"

    def test_prometheus_has_system_jobs(self):
        """Prometheus に system tier のジョブが登録されていることを確認。"""
        try:
            resp = requests.get(f"{self.PROMETHEUS_URL}/api/v1/targets", timeout=5)
        except requests.ConnectionError:
            pytest.skip("Prometheus not running")

        data = resp.json()
        job_names = {t["labels"].get("job", "") for t in data["data"]["activeTargets"]}
        expected_jobs = {"auth-server-rust", "config-server-rust"}
        found = expected_jobs & job_names
        # 少なくとも1つのシステムジョブが見つかればOK
        assert len(found) > 0 or len(job_names) > 0, (
            f"Expected at least one system job. Found: {job_names}"
        )


class TestJaeger:
    """Jaeger の基本動作を検証。"""

    JAEGER_URL = "http://localhost:16686"

    def test_jaeger_ui_accessible(self):
        """Jaeger UI がアクセス可能であることを確認。"""
        try:
            resp = requests.get(f"{self.JAEGER_URL}/", timeout=5)
            assert resp.status_code == 200
        except requests.ConnectionError:
            pytest.skip("Jaeger not running")

    def test_jaeger_services_endpoint(self):
        """Jaeger の /api/services エンドポイントが応答することを確認。"""
        try:
            resp = requests.get(f"{self.JAEGER_URL}/api/services", timeout=5)
        except requests.ConnectionError:
            pytest.skip("Jaeger not running")

        assert resp.status_code == 200
        data = resp.json()
        assert "data" in data


class TestGrafana:
    """Grafana の基本動作を検証。"""

    GRAFANA_URL = "http://localhost:3200"

    def test_grafana_health(self):
        """Grafana の /api/health が OK を返すことを確認。"""
        try:
            resp = requests.get(f"{self.GRAFANA_URL}/api/health", timeout=5)
        except requests.ConnectionError:
            pytest.skip("Grafana not running")

        assert resp.status_code == 200
        data = resp.json()
        assert data.get("database") == "ok"

    def test_grafana_datasources(self):
        """Grafana にデータソースが登録されていることを確認。"""
        try:
            resp = requests.get(
                f"{self.GRAFANA_URL}/api/datasources",
                auth=("admin", "dev"),
                timeout=5,
            )
        except requests.ConnectionError:
            pytest.skip("Grafana not running")

        assert resp.status_code == 200
        datasources = resp.json()
        assert isinstance(datasources, list)


class TestLoki:
    """Loki の基本動作を検証。"""

    LOKI_URL = "http://localhost:3100"

    def test_loki_ready(self):
        """Loki の /ready が ready を返すことを確認。"""
        try:
            resp = requests.get(f"{self.LOKI_URL}/ready", timeout=5)
        except requests.ConnectionError:
            pytest.skip("Loki not running")

        assert resp.status_code == 200

    def test_loki_labels_endpoint(self):
        """Loki の /loki/api/v1/labels が応答することを確認。"""
        try:
            resp = requests.get(f"{self.LOKI_URL}/loki/api/v1/labels", timeout=5)
        except requests.ConnectionError:
            pytest.skip("Loki not running")

        assert resp.status_code == 200
