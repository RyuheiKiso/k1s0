"""Session client abstraction."""

from __future__ import annotations

import uuid
from abc import ABC, abstractmethod
from datetime import datetime, timedelta, timezone
from enum import Enum, auto

from .models import CreateSessionRequest, RefreshSessionRequest, Session


class SessionError(Exception):
    """Session error."""

    class Code(Enum):
        NOT_FOUND = auto()
        REVOKED = auto()
        EXPIRED = auto()

    def __init__(self, message: str, code: SessionError.Code) -> None:
        super().__init__(message)
        self.code = code


class SessionClient(ABC):
    """Abstract session client."""

    @abstractmethod
    async def create(self, req: CreateSessionRequest) -> Session:
        ...

    @abstractmethod
    async def get(self, id: str) -> Session | None:
        ...

    @abstractmethod
    async def refresh(self, req: RefreshSessionRequest) -> Session:
        ...

    @abstractmethod
    async def revoke(self, id: str) -> None:
        ...

    @abstractmethod
    async def list_user_sessions(self, user_id: str) -> list[Session]:
        ...

    @abstractmethod
    async def revoke_all(self, user_id: str) -> int:
        ...


class InMemorySessionClient(SessionClient):
    """In-memory session client for testing."""

    def __init__(self) -> None:
        self._sessions: dict[str, Session] = {}

    async def create(self, req: CreateSessionRequest) -> Session:
        now = datetime.now(timezone.utc)
        session = Session(
            id=str(uuid.uuid4()),
            user_id=req.user_id,
            token=str(uuid.uuid4()),
            expires_at=now + timedelta(seconds=req.ttl_seconds),
            created_at=now,
            metadata=dict(req.metadata),
        )
        self._sessions[session.id] = session
        return session

    async def get(self, id: str) -> Session | None:
        return self._sessions.get(id)

    async def refresh(self, req: RefreshSessionRequest) -> Session:
        session = self._sessions.get(req.id)
        if session is None:
            raise SessionError(
                f"Session not found: {req.id}", SessionError.Code.NOT_FOUND
            )
        if session.revoked:
            raise SessionError(
                f"Session revoked: {req.id}", SessionError.Code.REVOKED
            )
        refreshed = Session(
            id=session.id,
            user_id=session.user_id,
            token=session.token,
            expires_at=datetime.now(timezone.utc) + timedelta(seconds=req.ttl_seconds),
            created_at=session.created_at,
            metadata=session.metadata,
        )
        self._sessions[req.id] = refreshed
        return refreshed

    async def revoke(self, id: str) -> None:
        session = self._sessions.get(id)
        if session is None:
            raise SessionError(
                f"Session not found: {id}", SessionError.Code.NOT_FOUND
            )
        self._sessions[id] = Session(
            id=session.id,
            user_id=session.user_id,
            token=session.token,
            expires_at=session.expires_at,
            created_at=session.created_at,
            revoked=True,
            metadata=session.metadata,
        )

    async def list_user_sessions(self, user_id: str) -> list[Session]:
        return [s for s in self._sessions.values() if s.user_id == user_id]

    async def revoke_all(self, user_id: str) -> int:
        count = 0
        for sid, session in list(self._sessions.items()):
            if session.user_id == user_id and not session.revoked:
                self._sessions[sid] = Session(
                    id=session.id,
                    user_id=session.user_id,
                    token=session.token,
                    expires_at=session.expires_at,
                    created_at=session.created_at,
                    revoked=True,
                    metadata=session.metadata,
                )
                count += 1
        return count
