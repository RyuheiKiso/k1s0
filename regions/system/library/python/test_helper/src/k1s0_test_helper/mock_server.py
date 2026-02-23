"""テスト用モックサーバー。"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class MockRoute:
    """モックルート定義。"""

    method: str
    path: str
    status: int
    body: str


class MockServer:
    """モックサーバー (インメモリ)。"""

    def __init__(self, routes: list[MockRoute]) -> None:
        self._routes = routes
        self._requests: list[tuple[str, str]] = []

    def handle(self, method: str, path: str) -> tuple[int, str] | None:
        """登録済みルートからレスポンスを取得する。"""
        self._requests.append((method, path))
        for route in self._routes:
            if route.method == method and route.path == path:
                return (route.status, route.body)
        return None

    @property
    def request_count(self) -> int:
        """記録されたリクエスト数を返す。"""
        return len(self._requests)

    @property
    def recorded_requests(self) -> list[tuple[str, str]]:
        """記録されたリクエストを返す。"""
        return list(self._requests)


class MockServerBuilder:
    """モックサーバービルダー。"""

    def __init__(self, server_type: str) -> None:
        self._server_type = server_type
        self._routes: list[MockRoute] = []

    @classmethod
    def notification_server(cls) -> MockServerBuilder:
        """Notification サーバーモックを構築する。"""
        return cls("notification")

    @classmethod
    def ratelimit_server(cls) -> MockServerBuilder:
        """Ratelimit サーバーモックを構築する。"""
        return cls("ratelimit")

    @classmethod
    def tenant_server(cls) -> MockServerBuilder:
        """Tenant サーバーモックを構築する。"""
        return cls("tenant")

    @property
    def server_type(self) -> str:
        return self._server_type

    def with_health_ok(self) -> MockServerBuilder:
        """ヘルスチェック用の成功レスポンスを追加する。"""
        self._routes.append(
            MockRoute(method="GET", path="/health", status=200, body='{"status":"ok"}')
        )
        return self

    def with_success_response(self, path: str, body: str) -> MockServerBuilder:
        """成功レスポンスルートを追加する。"""
        self._routes.append(MockRoute(method="POST", path=path, status=200, body=body))
        return self

    def with_error_response(self, path: str, status: int) -> MockServerBuilder:
        """エラーレスポンスルートを追加する。"""
        self._routes.append(
            MockRoute(method="POST", path=path, status=status, body='{"error":"mock error"}')
        )
        return self

    def build(self) -> MockServer:
        """モックサーバーを構築する。"""
        return MockServer(self._routes)
