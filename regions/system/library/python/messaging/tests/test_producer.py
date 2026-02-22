"""EventProducer のユニットテスト"""

from k1s0_messaging.models import EventEnvelope
from k1s0_messaging.noop import NoOpEventProducer


def test_noop_producer_publish() -> None:
    """NoOpEventProducer が発行済みメッセージを記録すること。"""
    producer = NoOpEventProducer()
    envelope = EventEnvelope(topic="events", payload=b"test")
    producer.publish(envelope)
    assert len(producer.published) == 1
    assert producer.published[0].topic == "events"


async def test_noop_producer_publish_async() -> None:
    """NoOpEventProducer が非同期発行を記録すること。"""
    producer = NoOpEventProducer()
    envelope = EventEnvelope(topic="events", payload=b"async")
    await producer.publish_async(envelope)
    assert len(producer.published) == 1


def test_noop_producer_publish_batch() -> None:
    """NoOpEventProducer がバッチ発行を記録すること。"""
    producer = NoOpEventProducer()
    envelopes = [
        EventEnvelope(topic="events", payload=b"msg1"),
        EventEnvelope(topic="events", payload=b"msg2"),
    ]
    producer.publish_batch(envelopes)
    assert len(producer.published) == 2


def test_noop_producer_context_manager() -> None:
    """コンテキストマネージャーで close が呼ばれること。"""
    with NoOpEventProducer() as producer:
        producer.publish(EventEnvelope(topic="events", payload=b"test"))
    assert producer.closed is True


def test_kafka_producer_publish_calls_confluent(mocker) -> None:
    """KafkaEventProducer が confluent_kafka.Producer を呼び出すこと。"""
    from k1s0_messaging.producer import KafkaEventProducer

    mock_producer = mocker.MagicMock()
    mocker.patch("confluent_kafka.Producer", return_value=mock_producer)

    producer = KafkaEventProducer(brokers=["localhost:9092"])
    envelope = EventEnvelope(topic="events", payload=b"data")
    producer.publish(envelope)

    mock_producer.produce.assert_called_once()
    mock_producer.flush.assert_called()


async def test_kafka_producer_publish_async(mocker) -> None:
    """KafkaEventProducer が非同期発行を実行すること。"""
    from k1s0_messaging.producer import KafkaEventProducer

    mock_producer = mocker.MagicMock()
    mocker.patch("confluent_kafka.Producer", return_value=mock_producer)

    producer = KafkaEventProducer(brokers=["localhost:9092"])
    envelope = EventEnvelope(topic="events", payload=b"async-data")
    await producer.publish_async(envelope)

    mock_producer.produce.assert_called_once()


def test_kafka_producer_publish_batch(mocker) -> None:
    """KafkaEventProducer がバッチ発行を実行すること。"""
    from k1s0_messaging.producer import KafkaEventProducer

    mock_producer = mocker.MagicMock()
    mocker.patch("confluent_kafka.Producer", return_value=mock_producer)

    producer = KafkaEventProducer(brokers=["localhost:9092"])
    envelopes = [
        EventEnvelope(topic="events", payload=b"msg1"),
        EventEnvelope(topic="events", payload=b"msg2"),
    ]
    producer.publish_batch(envelopes)

    assert mock_producer.produce.call_count == 2
    mock_producer.flush.assert_called()


def test_kafka_producer_close(mocker) -> None:
    """KafkaEventProducer の close がフラッシュして内部状態をクリアすること。"""
    from k1s0_messaging.producer import KafkaEventProducer

    mock_producer = mocker.MagicMock()
    mocker.patch("confluent_kafka.Producer", return_value=mock_producer)

    producer = KafkaEventProducer(brokers=["localhost:9092"])
    producer._get_producer()  # 内部プロデューサーを初期化
    producer.close()

    mock_producer.flush.assert_called()
    assert producer._producer is None


def test_kafka_producer_close_noop_if_not_initialized() -> None:
    """未初期化の KafkaEventProducer を close しても何も起きないこと。"""
    from k1s0_messaging.producer import KafkaEventProducer

    producer = KafkaEventProducer(brokers=["localhost:9092"])
    producer.close()  # エラーなく完了すること
    assert producer._producer is None


def test_kafka_producer_publish_raises_messaging_error(mocker) -> None:
    """produce 失敗時に MessagingError が発生すること。"""
    import pytest
    from k1s0_messaging.exceptions import MessagingError
    from k1s0_messaging.producer import KafkaEventProducer

    mock_producer = mocker.MagicMock()
    mock_producer.produce.side_effect = RuntimeError("broker down")
    mocker.patch("confluent_kafka.Producer", return_value=mock_producer)

    producer = KafkaEventProducer(brokers=["localhost:9092"])
    envelope = EventEnvelope(topic="events", payload=b"data")

    with pytest.raises(MessagingError) as exc_info:
        producer.publish(envelope)
    assert "PUBLISH_FAILED" in str(exc_info.value)


def test_kafka_producer_publish_batch_raises_messaging_error(mocker) -> None:
    """バッチ発行失敗時に MessagingError が発生すること。"""
    import pytest
    from k1s0_messaging.exceptions import MessagingError
    from k1s0_messaging.producer import KafkaEventProducer

    mock_producer = mocker.MagicMock()
    mock_producer.produce.side_effect = RuntimeError("broker down")
    mocker.patch("confluent_kafka.Producer", return_value=mock_producer)

    producer = KafkaEventProducer(brokers=["localhost:9092"])
    envelopes = [EventEnvelope(topic="events", payload=b"data")]

    with pytest.raises(MessagingError) as exc_info:
        producer.publish_batch(envelopes)
    assert "PUBLISH_FAILED" in str(exc_info.value)


def test_kafka_producer_publish_with_headers(mocker) -> None:
    """ヘッダー付きメッセージを発行できること。"""
    from k1s0_messaging.producer import KafkaEventProducer

    mock_producer = mocker.MagicMock()
    mocker.patch("confluent_kafka.Producer", return_value=mock_producer)

    producer = KafkaEventProducer(brokers=["localhost:9092"])
    envelope = EventEnvelope(
        topic="events",
        payload=b"data",
        headers={"trace-id": "abc123"},
    )
    producer.publish(envelope)

    call_kwargs = mock_producer.produce.call_args
    assert call_kwargs is not None
