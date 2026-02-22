"""Kafka ヘルスチェック"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from enum import StrEnum
from typing import Any

from confluent_kafka.admin import AdminClient


class HealthStatus(StrEnum):
    """ヘルスチェックステータス。"""

    HEALTHY = "HEALTHY"
    UNHEALTHY = "UNHEALTHY"


@dataclass
class HealthCheckResult:
    """ヘルスチェック結果。"""

    status: HealthStatus
    message: str = ""
    details: dict[str, Any] | None = None


class KafkaHealthCheck:
    """Kafka ブローカーへの接続ヘルスチェック。"""

    def __init__(self, brokers: list[str], timeout_seconds: float = 5.0) -> None:
        self._brokers = brokers
        self._timeout_seconds = timeout_seconds

    def check(self) -> HealthCheckResult:
        """同期ヘルスチェックを実行する。"""
        try:
            admin = AdminClient({"bootstrap.servers": ",".join(self._brokers)})
            metadata = admin.list_topics(timeout=self._timeout_seconds)
            broker_count = len(metadata.brokers)
            return HealthCheckResult(
                status=HealthStatus.HEALTHY,
                message=f"Connected to {broker_count} broker(s)",
                details={"broker_count": broker_count},
            )
        except Exception as e:
            return HealthCheckResult(
                status=HealthStatus.UNHEALTHY,
                message=f"Failed to connect to Kafka: {e}",
            )

    async def check_async(self) -> HealthCheckResult:
        """非同期ヘルスチェックを実行する。"""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self.check)
