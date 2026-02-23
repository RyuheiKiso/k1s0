"""WebSocket types."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum, auto


class MessageType(Enum):
    """WebSocket message type."""

    TEXT = auto()
    BINARY = auto()
    PING = auto()
    PONG = auto()
    CLOSE = auto()


@dataclass
class WsMessage:
    """WebSocket message."""

    type: MessageType
    payload: str | bytes = b""

    @staticmethod
    def text(s: str) -> WsMessage:
        return WsMessage(type=MessageType.TEXT, payload=s)

    @staticmethod
    def binary(data: bytes) -> WsMessage:
        return WsMessage(type=MessageType.BINARY, payload=data)


class ConnectionState(Enum):
    """WebSocket connection state."""

    DISCONNECTED = auto()
    CONNECTING = auto()
    CONNECTED = auto()
    RECONNECTING = auto()
    CLOSING = auto()
