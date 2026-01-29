"""Readiness check with dependency verification."""

from __future__ import annotations

import logging
from collections.abc import Awaitable, Callable
from typing import Any

logger = logging.getLogger("k1s0.health.readiness")

DependencyCheck = Callable[[], Awaitable[bool]]


class ReadinessChecker:
    """Manages readiness checks for service dependencies.

    Register dependency checks (e.g., database, cache, external services)
    and run them all to determine overall readiness.
    """

    def __init__(self) -> None:
        self._checks: dict[str, DependencyCheck] = {}

    def register(self, name: str, check: DependencyCheck) -> None:
        """Register a named dependency check.

        Args:
            name: Human-readable name of the dependency.
            check: Async callable returning True if the dependency is healthy.
        """
        self._checks[name] = check

    async def check(self) -> dict[str, Any]:
        """Run all registered dependency checks.

        Returns:
            A dict with overall "status" ("ok" or "unavailable") and
            per-dependency results in "checks".
        """
        results: dict[str, str] = {}
        all_ok = True

        for name, check_fn in self._checks.items():
            try:
                healthy = await check_fn()
                results[name] = "ok" if healthy else "failed"
                if not healthy:
                    all_ok = False
            except Exception:
                logger.exception("Readiness check '%s' raised an exception", name)
                results[name] = "error"
                all_ok = False

        return {
            "status": "ok" if all_ok else "unavailable",
            "checks": results,
        }
