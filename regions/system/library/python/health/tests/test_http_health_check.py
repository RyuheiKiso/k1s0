"""HttpHealthCheck unit tests."""

from http.server import HTTPServer, BaseHTTPRequestHandler
import threading

import pytest

from k1s0_health import HealthChecker, HealthStatus
from k1s0_health.http_health_check import HttpHealthCheck


class _OkHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        self.send_response(200)
        self.end_headers()

    def log_message(self, *args: object) -> None:  # noqa: ARG002
        pass  # suppress logs


class _ErrorHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        self.send_response(503)
        self.end_headers()

    def log_message(self, *args: object) -> None:  # noqa: ARG002
        pass


def _start_server(handler: type) -> tuple[HTTPServer, str]:
    server = HTTPServer(("127.0.0.1", 0), handler)
    port = server.server_address[1]
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, f"http://127.0.0.1:{port}"


def test_default_name() -> None:
    check = HttpHealthCheck("http://example.com/healthz")
    assert check.name == "http"


def test_custom_name() -> None:
    check = HttpHealthCheck("http://example.com/healthz", name="upstream")
    assert check.name == "upstream"


async def test_healthy() -> None:
    server, url = _start_server(_OkHandler)
    try:
        check = HttpHealthCheck(url, name="test")
        await check.check()  # should not raise
    finally:
        server.shutdown()


async def test_unhealthy_status() -> None:
    server, url = _start_server(_ErrorHandler)
    try:
        check = HttpHealthCheck(url, name="test")
        with pytest.raises(RuntimeError, match="status 503"):
            await check.check()
    finally:
        server.shutdown()


async def test_connection_refused() -> None:
    check = HttpHealthCheck(
        "http://127.0.0.1:1/healthz",
        timeout_seconds=1.0,
        name="unreachable",
    )
    with pytest.raises(RuntimeError, match="HTTP check failed"):
        await check.check()


async def test_integration_with_checker() -> None:
    server, url = _start_server(_OkHandler)
    try:
        checker = HealthChecker()
        checker.add(HttpHealthCheck(url, name="upstream"))
        resp = await checker.run_all()
        assert resp.status == HealthStatus.HEALTHY
        assert resp.checks["upstream"].status == HealthStatus.HEALTHY
    finally:
        server.shutdown()
