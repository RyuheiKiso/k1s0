"""WebSocket client abstraction."""

from __future__ import annotations

import asyncio
from abc import ABC, abstractmethod
from enum import Enum, auto

from .types import ConnectionState, WsMessage


class WsError(Exception):
    """WebSocket error."""

    class Code(Enum):
        NOT_CONNECTED = auto()
        NO_MESSAGES = auto()
        ALREADY_CONNECTED = auto()
        CONNECTION_FAILED = auto()

    def __init__(self, message: str, code: WsError.Code) -> None:
        super().__init__(message)
        self.code = code


class WsClient(ABC):
    """Abstract WebSocket client."""

    @abstractmethod
    async def connect(self) -> None:
        ...

    @abstractmethod
    async def disconnect(self) -> None:
        ...

    @abstractmethod
    async def send(self, message: WsMessage) -> None:
        ...

    @abstractmethod
    async def receive(self) -> WsMessage:
        ...

    @property
    @abstractmethod
    def state(self) -> ConnectionState:
        ...


class InMemoryWsClient(WsClient):
    """In-memory WebSocket client for testing."""

    def __init__(self) -> None:
        self._state = ConnectionState.DISCONNECTED
        self._recv_queue: asyncio.Queue[WsMessage] = asyncio.Queue()
        self._sent_messages: list[WsMessage] = []

    @property
    def state(self) -> ConnectionState:
        return self._state

    async def connect(self) -> None:
        if self._state in (ConnectionState.CONNECTED, ConnectionState.CONNECTING):
            raise WsError("Already connected", WsError.Code.ALREADY_CONNECTED)
        self._state = ConnectionState.CONNECTING
        self._state = ConnectionState.CONNECTED

    async def disconnect(self) -> None:
        self._state = ConnectionState.CLOSING
        self._state = ConnectionState.DISCONNECTED

    async def send(self, message: WsMessage) -> None:
        if self._state != ConnectionState.CONNECTED:
            raise WsError("Not connected", WsError.Code.NOT_CONNECTED)
        self._sent_messages.append(message)

    async def receive(self) -> WsMessage:
        if self._state != ConnectionState.CONNECTED:
            raise WsError("Not connected", WsError.Code.NOT_CONNECTED)
        if self._recv_queue.empty():
            raise WsError("No messages available", WsError.Code.NO_MESSAGES)
        return self._recv_queue.get_nowait()

    def inject_message(self, msg: WsMessage) -> None:
        self._recv_queue.put_nowait(msg)

    def get_sent_messages(self) -> list[WsMessage]:
        return list(self._sent_messages)
