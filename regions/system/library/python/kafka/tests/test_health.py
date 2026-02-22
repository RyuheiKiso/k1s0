"""KafkaHealthCheck のユニットテスト（confluent-kafka モック）"""

from unittest.mock import MagicMock, patch

import pytest
from k1s0_kafka.health import HealthStatus, KafkaHealthCheck


def test_health_check_healthy(mocker) -> None:
    """ブローカー接続成功時に HEALTHY が返ること。"""
    mock_metadata = MagicMock()
    mock_metadata.brokers = {1: MagicMock(), 2: MagicMock()}
    mock_admin = MagicMock()
    mock_admin.list_topics.return_value = mock_metadata

    with patch("k1s0_kafka.health.AdminClient", return_value=mock_admin):
        checker = KafkaHealthCheck(["localhost:9092"])
        result = checker.check()

    assert result.status == HealthStatus.HEALTHY
    assert result.details is not None
    assert result.details["broker_count"] == 2


def test_health_check_unhealthy(mocker) -> None:
    """ブローカー接続失敗時に UNHEALTHY が返ること。"""
    with patch("k1s0_kafka.health.AdminClient", side_effect=Exception("Connection refused")):
        checker = KafkaHealthCheck(["localhost:9092"])
        result = checker.check()

    assert result.status == HealthStatus.UNHEALTHY
    assert "Connection refused" in result.message


@pytest.mark.asyncio
async def test_health_check_async_healthy(mocker) -> None:
    """非同期ヘルスチェックで HEALTHY が返ること。"""
    mock_metadata = MagicMock()
    mock_metadata.brokers = {1: MagicMock()}
    mock_admin = MagicMock()
    mock_admin.list_topics.return_value = mock_metadata

    with patch("k1s0_kafka.health.AdminClient", return_value=mock_admin):
        checker = KafkaHealthCheck(["localhost:9092"])
        result = await checker.check_async()

    assert result.status == HealthStatus.HEALTHY
