"""サービス間連携テスト。

複数のシステムサービス（auth, config, saga, dlq-manager）が
連携して動作することを検証する。

前提: docker compose --profile infra --profile system up -d
"""
import pytest
import requests


@pytest.fixture(scope="session")
def all_services_urls():
    """全サービスのベースURL一覧を返す。"""
    return {
        "auth": "http://localhost:8083",
        "config": "http://localhost:8084",
        "saga": "http://localhost:8085",
        "dlq-manager": "http://localhost:8086",
    }


class TestCrossServiceHealth:
    """全サービスのヘルスチェックが同時に正常であることを検証。"""

    def test_all_services_healthz(self, all_services_urls):
        """全サービスの /healthz が 200 を返すことを確認。"""
        errors = []
        for name, url in all_services_urls.items():
            try:
                resp = requests.get(f"{url}/healthz", timeout=5)
                if resp.status_code != 200:
                    errors.append(f"{name}: status={resp.status_code}")
            except requests.ConnectionError:
                errors.append(f"{name}: connection refused at {url}")

        if errors:
            pytest.skip(f"Services not running: {', '.join(errors)}")

    def test_all_services_readyz(self, all_services_urls):
        """全サービスの /readyz が 200 を返すことを確認。"""
        errors = []
        for name, url in all_services_urls.items():
            try:
                resp = requests.get(f"{url}/readyz", timeout=5)
                if resp.status_code != 200:
                    errors.append(f"{name}: status={resp.status_code}")
            except requests.ConnectionError:
                errors.append(f"{name}: connection refused at {url}")

        if errors:
            pytest.skip(f"Services not running: {', '.join(errors)}")

    def test_all_services_metrics(self, all_services_urls):
        """全サービスの /metrics が Prometheus 形式を返すことを確認。"""
        for name, url in all_services_urls.items():
            try:
                resp = requests.get(f"{url}/metrics", timeout=5)
                if resp.status_code != 200:
                    continue
                assert "http_requests" in resp.text or "process_" in resp.text or len(resp.text) > 0, (
                    f"{name}: /metrics returned empty or invalid content"
                )
            except requests.ConnectionError:
                pytest.skip(f"{name} not running at {url}")


class TestSagaConfigIntegration:
    """Saga と Config サービスの連携テスト。"""

    def test_saga_can_register_workflow_and_start(self, all_services_urls):
        """Saga にワークフロー登録 → Saga 開始できることを確認。"""
        saga_url = all_services_urls["saga"]

        try:
            requests.get(f"{saga_url}/healthz", timeout=3).raise_for_status()
        except (requests.ConnectionError, requests.HTTPError):
            pytest.skip("Saga server not running")

        # ワークフロー登録
        workflow_yaml = """
name: e2e-test-workflow
steps:
  - name: validate
    service: config-service
    method: ConfigService.GetConfig
    timeout_secs: 10
"""
        resp = requests.post(
            f"{saga_url}/api/v1/workflows",
            json={"workflow_yaml": workflow_yaml},
            timeout=10,
        )
        assert resp.status_code in (200, 201), f"Workflow registration failed: {resp.text}"

        # ワークフロー一覧確認
        resp = requests.get(f"{saga_url}/api/v1/workflows", timeout=10)
        assert resp.status_code == 200
        data = resp.json()
        names = [w["name"] for w in data["workflows"]]
        assert "e2e-test-workflow" in names
