"""EventConsumer のユニットテスト"""

import pytest
from k1s0_messaging.consumer import KafkaEventConsumer
from k1s0_messaging.exceptions import MessagingError
from k1s0_messaging.models import ConsumedMessage, ConsumerConfig


def test_kafka_consumer_receive_none_on_timeout(mocker) -> None:
    """タイムアウト時に None が返ること。"""
    from k1s0_messaging.consumer import KafkaEventConsumer
    from k1s0_messaging.models import ConsumerConfig

    mock_consumer = mocker.MagicMock()
    mock_consumer.poll.return_value = None
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)
    result = consumer.receive(timeout_seconds=0.1)
    assert result is None


def test_kafka_consumer_receive_message(mocker) -> None:
    """メッセージ受信が ConsumedMessage を返すこと。"""
    from k1s0_messaging.consumer import KafkaEventConsumer
    from k1s0_messaging.models import ConsumerConfig

    mock_msg = mocker.MagicMock()
    mock_msg.error.return_value = None
    mock_msg.topic.return_value = "events"
    mock_msg.partition.return_value = 0
    mock_msg.offset.return_value = 42
    mock_msg.value.return_value = b"payload"
    mock_msg.key.return_value = None
    mock_msg.headers.return_value = None

    mock_consumer = mocker.MagicMock()
    mock_consumer.poll.return_value = mock_msg
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)
    result = consumer.receive()

    assert result is not None
    assert result.topic == "events"
    assert result.offset == 42
    assert result.payload == b"payload"


def test_kafka_consumer_receive_message_with_headers(mocker) -> None:
    """ヘッダー付きメッセージを受信できること。"""
    mock_msg = mocker.MagicMock()
    mock_msg.error.return_value = None
    mock_msg.topic.return_value = "events"
    mock_msg.partition.return_value = 1
    mock_msg.offset.return_value = 10
    mock_msg.value.return_value = b"with-headers"
    mock_msg.key.return_value = b"key1"
    mock_msg.headers.return_value = [("trace-id", b"abc123")]

    mock_consumer = mocker.MagicMock()
    mock_consumer.poll.return_value = mock_msg
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)
    result = consumer.receive()

    assert result is not None
    assert result.headers == {"trace-id": "abc123"}
    assert result.key == b"key1"


def test_kafka_consumer_receive_kafka_error(mocker) -> None:
    """Kafka エラー発生時に MessagingError が発生すること。"""
    mock_error = mocker.MagicMock()
    mock_error.__bool__ = lambda self: True
    mock_error.__str__ = lambda self: "OFFSET_OUT_OF_RANGE"

    mock_msg = mocker.MagicMock()
    mock_msg.error.return_value = mock_error

    mock_consumer = mocker.MagicMock()
    mock_consumer.poll.return_value = mock_msg
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)

    with pytest.raises(MessagingError) as exc_info:
        consumer.receive()
    assert "RECEIVE_FAILED" in str(exc_info.value)


def test_kafka_consumer_receive_unexpected_exception(mocker) -> None:
    """予期しない例外が MessagingError にラップされること。"""
    mock_consumer = mocker.MagicMock()
    mock_consumer.poll.side_effect = RuntimeError("connection lost")
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)

    with pytest.raises(MessagingError) as exc_info:
        consumer.receive()
    assert "RECEIVE_FAILED" in str(exc_info.value)


def test_kafka_consumer_subscribe(mocker) -> None:
    """subscribe がコンシューマーの subscribe を呼び出すこと。"""
    mock_consumer = mocker.MagicMock()
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)
    consumer.subscribe(["events", "orders"])

    mock_consumer.subscribe.assert_called_once_with(["events", "orders"])


def test_kafka_consumer_commit(mocker) -> None:
    """commit がオフセット+1 でコミットすること。"""
    mock_consumer = mocker.MagicMock()
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)
    mock_tp = mocker.MagicMock()
    mocker.patch("confluent_kafka.TopicPartition", return_value=mock_tp)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)
    msg = ConsumedMessage(topic="events", partition=0, offset=42, payload=b"data")
    consumer.commit(msg)

    mock_consumer.commit.assert_called_once()


async def test_kafka_consumer_receive_async(mocker) -> None:
    """receive_async が ConsumedMessage を返すこと。"""
    mock_msg = mocker.MagicMock()
    mock_msg.error.return_value = None
    mock_msg.topic.return_value = "async-events"
    mock_msg.partition.return_value = 0
    mock_msg.offset.return_value = 5
    mock_msg.value.return_value = b"async-payload"
    mock_msg.key.return_value = None
    mock_msg.headers.return_value = None

    mock_consumer = mocker.MagicMock()
    mock_consumer.poll.return_value = mock_msg
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)
    result = await consumer.receive_async()

    assert result is not None
    assert result.topic == "async-events"


def test_kafka_consumer_close(mocker) -> None:
    """close がコンシューマーを閉じて内部状態をクリアすること。"""
    mock_consumer = mocker.MagicMock()
    mocker.patch("confluent_kafka.Consumer", return_value=mock_consumer)

    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)
    consumer._get_consumer()  # 内部コンシューマーを初期化
    consumer.close()

    mock_consumer.close.assert_called_once()
    assert consumer._consumer is None


def test_kafka_consumer_close_noop_if_not_initialized() -> None:
    """未初期化の KafkaEventConsumer を close しても何も起きないこと。"""
    config = ConsumerConfig(brokers=["localhost:9092"], group_id="test-group")
    consumer = KafkaEventConsumer(config)
    consumer.close()  # エラーなく完了すること
    assert consumer._consumer is None
