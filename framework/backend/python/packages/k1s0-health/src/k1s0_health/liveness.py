"""Liveness check implementation."""

from __future__ import annotations


def liveness_check() -> dict[str, str]:
    """Perform a basic liveness check.

    Simply returns OK to indicate the process is alive and responsive.

    Returns:
        A dict with status "ok".
    """
    return {"status": "ok"}
