"""health library unit tests."""

from k1s0_health import HealthCheck, HealthChecker, HealthStatus


class _AlwaysHealthy(HealthCheck):
    @property
    def name(self) -> str:
        return "always_healthy"

    async def check(self) -> None:
        pass


class _AlwaysUnhealthy(HealthCheck):
    @property
    def name(self) -> str:
        return "always_unhealthy"

    async def check(self) -> None:
        raise RuntimeError("down")


async def test_all_healthy() -> None:
    checker = HealthChecker()
    checker.add(_AlwaysHealthy())
    resp = await checker.run_all()
    assert resp.status == HealthStatus.HEALTHY
    assert resp.checks["always_healthy"].status == HealthStatus.HEALTHY


async def test_one_unhealthy() -> None:
    checker = HealthChecker()
    checker.add(_AlwaysHealthy())
    checker.add(_AlwaysUnhealthy())
    resp = await checker.run_all()
    assert resp.status == HealthStatus.UNHEALTHY
    assert resp.checks["always_healthy"].status == HealthStatus.HEALTHY
    assert resp.checks["always_unhealthy"].status == HealthStatus.UNHEALTHY
    assert resp.checks["always_unhealthy"].message == "down"


async def test_no_checks() -> None:
    checker = HealthChecker()
    resp = await checker.run_all()
    assert resp.status == HealthStatus.HEALTHY
    assert resp.checks == {}


async def test_response_has_timestamp() -> None:
    checker = HealthChecker()
    resp = await checker.run_all()
    assert resp.timestamp is not None
