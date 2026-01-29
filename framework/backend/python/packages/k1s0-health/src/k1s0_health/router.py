"""FastAPI health check router."""

from __future__ import annotations

from typing import Any

from fastapi import APIRouter
from fastapi.responses import JSONResponse

from k1s0_health.liveness import liveness_check
from k1s0_health.readiness import ReadinessChecker

health_router = APIRouter(tags=["health"])

_readiness_checker = ReadinessChecker()


@health_router.get("/healthz")
async def healthz() -> dict[str, str]:
    """Liveness probe endpoint.

    Returns HTTP 200 if the process is alive.
    """
    return liveness_check()


@health_router.get("/readyz")
async def readyz() -> Any:
    """Readiness probe endpoint.

    Returns HTTP 200 if all dependency checks pass,
    or HTTP 503 with details if any check fails.
    """
    result = await _readiness_checker.check()
    if result["status"] == "ok":
        return result
    return JSONResponse(status_code=503, content=result)


def get_readiness_checker() -> ReadinessChecker:
    """Get the global readiness checker to register dependency checks."""
    return _readiness_checker
