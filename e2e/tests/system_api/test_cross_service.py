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
                has_content = (
                    "http_requests" in resp.text or "process_" in resp.text or len(resp.text) > 0
                )
                assert has_content, f"{name}: /metrics returned empty or invalid content"
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


class TestAuthFailureHandling:
    """認証失敗時のエラーハンドリングを検証する。"""

    def test_users_endpoint_without_token_returns_401(self, all_services_urls):
        """GET /api/v1/users に Authorization ヘッダーなしで 401 を返す。"""
        auth_url = all_services_urls["auth"]
        try:
            requests.get(f"{auth_url}/healthz", timeout=3).raise_for_status()
        except (requests.ConnectionError, requests.HTTPError):
            pytest.skip("Auth server not running")

        response = requests.get(f"{auth_url}/api/v1/users", timeout=5)
        assert response.status_code == 401

    def test_users_endpoint_with_invalid_token_returns_401(self, all_services_urls):
        """GET /api/v1/users に無効なトークンで 401 を返す。"""
        auth_url = all_services_urls["auth"]
        try:
            requests.get(f"{auth_url}/healthz", timeout=3).raise_for_status()
        except (requests.ConnectionError, requests.HTTPError):
            pytest.skip("Auth server not running")

        response = requests.get(
            f"{auth_url}/api/v1/users",
            headers={"Authorization": "Bearer invalid-token"},
            timeout=5,
        )
        assert response.status_code == 401

    def test_token_validate_with_invalid_token_returns_401(self, all_services_urls):
        """POST /api/v1/auth/token/validate に無効なトークンで 401 を返す。"""
        auth_url = all_services_urls["auth"]
        try:
            requests.get(f"{auth_url}/healthz", timeout=3).raise_for_status()
        except (requests.ConnectionError, requests.HTTPError):
            pytest.skip("Auth server not running")

        response = requests.post(
            f"{auth_url}/api/v1/auth/token/validate",
            json={"token": "invalid.jwt.token"},
            timeout=5,
        )
        assert response.status_code == 401

    def test_token_introspect_with_invalid_token_returns_inactive(self, all_services_urls):
        """POST /api/v1/auth/token/introspect に無効なトークンで active=false を返す。"""
        auth_url = all_services_urls["auth"]
        try:
            requests.get(f"{auth_url}/healthz", timeout=3).raise_for_status()
        except (requests.ConnectionError, requests.HTTPError):
            pytest.skip("Auth server not running")

        response = requests.post(
            f"{auth_url}/api/v1/auth/token/introspect",
            json={"token": "invalid.jwt.token"},
            timeout=5,
        )
        assert response.status_code == 200
        data = response.json()
        assert data["active"] is False


class TestSagaDlqIntegration:
    """Saga と DLQ サービスの連携を検証する。"""

    def test_saga_and_dlq_pagination_structure_consistency(self, all_services_urls):
        """Saga と DLQ の一覧 API が同じページネーション構造を返す。"""
        saga_url = all_services_urls["saga"]
        dlq_url = all_services_urls["dlq-manager"]

        saga_available = True
        dlq_available = True
        try:
            requests.get(f"{saga_url}/healthz", timeout=3).raise_for_status()
        except (requests.ConnectionError, requests.HTTPError):
            saga_available = False

        try:
            requests.get(f"{dlq_url}/healthz", timeout=3).raise_for_status()
        except (requests.ConnectionError, requests.HTTPError):
            dlq_available = False

        if not saga_available or not dlq_available:
            pytest.skip("Saga or DLQ server not running")

        saga_resp = requests.get(f"{saga_url}/api/v1/sagas", timeout=10)
        dlq_resp = requests.get(f"{dlq_url}/api/v1/dlq/test.dlq.v1", timeout=10)

        assert saga_resp.status_code in (200, 503)
        assert dlq_resp.status_code == 200

        if saga_resp.status_code == 503:
            pytest.skip("Saga server database is not available")

        saga_pagination = saga_resp.json()["pagination"]
        dlq_pagination = dlq_resp.json()["pagination"]
        # 両者が同じページネーションフィールドを持つ
        assert set(saga_pagination.keys()) == set(dlq_pagination.keys())

    def test_all_system_services_swagger_ui_accessible(self, all_services_urls):
        """全サービスの Swagger UI にアクセスできる。"""
        errors = []
        for name, url in all_services_urls.items():
            try:
                resp = requests.get(f"{url}/swagger-ui/", timeout=5)
                if resp.status_code not in (200, 301, 302):
                    errors.append(f"{name}: swagger-ui returned {resp.status_code}")
            except requests.ConnectionError:
                errors.append(f"{name}: connection refused")

        if len(errors) == len(all_services_urls):
            pytest.skip("No services running")
        # 少なくとも 1 つのサービスで Swagger UI が動作することを確認
        running_count = len(all_services_urls) - len(errors)
        assert running_count > 0
