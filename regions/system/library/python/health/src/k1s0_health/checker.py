"""Health check framework."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum


class HealthStatus(str, Enum):
    """Health status enum."""

    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"


@dataclass
class CheckResult:
    """Result of a single health check."""

    status: HealthStatus
    message: str | None = None


@dataclass
class HealthResponse:
    """Aggregated health check response."""

    status: HealthStatus
    checks: dict[str, CheckResult] = field(default_factory=dict)
    timestamp: datetime = field(
        default_factory=lambda: datetime.now(timezone.utc)
    )


class HealthCheck(ABC):
    """Abstract health check."""

    @property
    @abstractmethod
    def name(self) -> str: ...

    @abstractmethod
    async def check(self) -> None: ...


class HealthChecker:
    """Runs multiple health checks and aggregates results."""

    def __init__(self) -> None:
        self._checks: list[HealthCheck] = []

    def add(self, check: HealthCheck) -> None:
        """Register a health check."""
        self._checks.append(check)

    async def run_all(self) -> HealthResponse:
        """Run all registered checks."""
        results: dict[str, CheckResult] = {}
        overall = HealthStatus.HEALTHY
        for c in self._checks:
            try:
                await c.check()
                results[c.name] = CheckResult(status=HealthStatus.HEALTHY)
            except Exception as e:
                results[c.name] = CheckResult(
                    status=HealthStatus.UNHEALTHY, message=str(e)
                )
                overall = HealthStatus.UNHEALTHY
        return HealthResponse(status=overall, checks=results)

    async def readyz(self) -> HealthResponse:
        """Run all registered checks (alias for run_all)."""
        return await self.run_all()

    def healthz(self) -> dict[str, str]:
        """Return a simple liveness response."""
        return {"status": "ok"}
