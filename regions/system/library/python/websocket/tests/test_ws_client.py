"""websocket library unit tests."""

import pytest

from k1s0_websocket import (
    ConnectionState,
    InMemoryWsClient,
    MessageType,
    WsConfig,
    WsError,
    WsMessage,
)


async def test_connect_and_disconnect() -> None:
    client = InMemoryWsClient()
    assert client.state == ConnectionState.DISCONNECTED

    await client.connect()
    assert client.state == ConnectionState.CONNECTED

    await client.disconnect()
    assert client.state == ConnectionState.DISCONNECTED


async def test_send_and_receive() -> None:
    client = InMemoryWsClient()
    await client.connect()

    client.inject_message(WsMessage.text("hello"))
    msg = await client.receive()
    assert msg.type == MessageType.TEXT
    assert msg.payload == "hello"

    await client.send(WsMessage.text("world"))
    sent = client.get_sent_messages()
    assert len(sent) == 1


async def test_send_while_disconnected() -> None:
    client = InMemoryWsClient()
    with pytest.raises(WsError) as exc_info:
        await client.send(WsMessage.text("hello"))
    assert exc_info.value.code == WsError.Code.NOT_CONNECTED


async def test_receive_while_disconnected() -> None:
    client = InMemoryWsClient()
    with pytest.raises(WsError) as exc_info:
        await client.receive()
    assert exc_info.value.code == WsError.Code.NOT_CONNECTED


async def test_receive_empty_queue() -> None:
    client = InMemoryWsClient()
    await client.connect()
    with pytest.raises(WsError) as exc_info:
        await client.receive()
    assert exc_info.value.code == WsError.Code.NO_MESSAGES


async def test_double_connect() -> None:
    client = InMemoryWsClient()
    await client.connect()
    with pytest.raises(WsError) as exc_info:
        await client.connect()
    assert exc_info.value.code == WsError.Code.ALREADY_CONNECTED


async def test_text_factory() -> None:
    msg = WsMessage.text("hello")
    assert msg.type == MessageType.TEXT
    assert msg.payload == "hello"


async def test_binary_factory() -> None:
    msg = WsMessage.binary(b"\x01\x02\x03")
    assert msg.type == MessageType.BINARY
    assert msg.payload == b"\x01\x02\x03"


async def test_config_defaults() -> None:
    config = WsConfig()
    assert config.url == "ws://localhost"
    assert config.reconnect is True
    assert config.max_reconnect_attempts == 5
    assert config.ping_interval_ms is None


async def test_connection_states() -> None:
    states = list(ConnectionState)
    assert len(states) == 5


async def test_message_types() -> None:
    types = list(MessageType)
    assert len(types) == 5
