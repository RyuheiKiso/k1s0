"""WebSocket configuration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class WsConfig:
    """WebSocket client configuration."""

    url: str = "ws://localhost"
    reconnect: bool = True
    max_reconnect_attempts: int = 5
    reconnect_delay_ms: int = 1000
    ping_interval_ms: int | None = None
