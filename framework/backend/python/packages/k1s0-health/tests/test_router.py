"""Tests for health check endpoints."""

from __future__ import annotations

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

from k1s0_health.liveness import liveness_check
from k1s0_health.readiness import ReadinessChecker
from k1s0_health.router import health_router


@pytest.fixture()
def client() -> TestClient:
    app = FastAPI()
    app.include_router(health_router)
    return TestClient(app)


class TestLiveness:
    def test_liveness_returns_ok(self) -> None:
        result = liveness_check()
        assert result == {"status": "ok"}

    def test_healthz_endpoint(self, client: TestClient) -> None:
        response = client.get("/healthz")
        assert response.status_code == 200
        assert response.json() == {"status": "ok"}


class TestReadiness:
    @pytest.mark.anyio()
    async def test_no_checks_returns_ok(self) -> None:
        checker = ReadinessChecker()
        result = await checker.check()
        assert result["status"] == "ok"
        assert result["checks"] == {}

    @pytest.mark.anyio()
    async def test_passing_check(self) -> None:
        checker = ReadinessChecker()

        async def healthy() -> bool:
            return True

        checker.register("db", healthy)
        result = await checker.check()
        assert result["status"] == "ok"
        assert result["checks"]["db"] == "ok"

    @pytest.mark.anyio()
    async def test_failing_check(self) -> None:
        checker = ReadinessChecker()

        async def unhealthy() -> bool:
            return False

        checker.register("db", unhealthy)
        result = await checker.check()
        assert result["status"] == "unavailable"
        assert result["checks"]["db"] == "failed"

    @pytest.mark.anyio()
    async def test_erroring_check(self) -> None:
        checker = ReadinessChecker()

        async def broken() -> bool:
            msg = "connection refused"
            raise ConnectionError(msg)

        checker.register("cache", broken)
        result = await checker.check()
        assert result["status"] == "unavailable"
        assert result["checks"]["cache"] == "error"

    def test_readyz_endpoint_ok(self, client: TestClient) -> None:
        response = client.get("/readyz")
        assert response.status_code == 200
