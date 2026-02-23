"""session_client library unit tests."""

import pytest

from k1s0_session_client import (
    CreateSessionRequest,
    InMemorySessionClient,
    RefreshSessionRequest,
    Session,
    SessionClient,
    SessionError,
)


async def test_create_session() -> None:
    client = InMemorySessionClient()
    req = CreateSessionRequest(user_id="user-1", ttl_seconds=3600)
    session = await client.create(req)
    assert session.user_id == "user-1"
    assert session.id
    assert session.token
    assert session.revoked is False


async def test_get_session() -> None:
    client = InMemorySessionClient()
    req = CreateSessionRequest(user_id="user-1", ttl_seconds=3600)
    created = await client.create(req)

    retrieved = await client.get(created.id)
    assert retrieved is not None
    assert retrieved.id == created.id
    assert retrieved.user_id == "user-1"


async def test_get_not_found() -> None:
    client = InMemorySessionClient()
    result = await client.get("nonexistent")
    assert result is None


async def test_refresh_session() -> None:
    client = InMemorySessionClient()
    req = CreateSessionRequest(user_id="user-1", ttl_seconds=60)
    created = await client.create(req)

    refresh_req = RefreshSessionRequest(id=created.id, ttl_seconds=7200)
    refreshed = await client.refresh(refresh_req)
    assert refreshed.id == created.id
    assert refreshed.expires_at > created.expires_at


async def test_refresh_not_found() -> None:
    client = InMemorySessionClient()
    req = RefreshSessionRequest(id="nonexistent", ttl_seconds=3600)
    with pytest.raises(SessionError) as exc_info:
        await client.refresh(req)
    assert exc_info.value.code == SessionError.Code.NOT_FOUND


async def test_revoke_session() -> None:
    client = InMemorySessionClient()
    req = CreateSessionRequest(user_id="user-1", ttl_seconds=3600)
    created = await client.create(req)

    await client.revoke(created.id)
    revoked = await client.get(created.id)
    assert revoked is not None
    assert revoked.revoked is True


async def test_refresh_revoked() -> None:
    client = InMemorySessionClient()
    req = CreateSessionRequest(user_id="user-1", ttl_seconds=3600)
    created = await client.create(req)
    await client.revoke(created.id)

    refresh_req = RefreshSessionRequest(id=created.id, ttl_seconds=3600)
    with pytest.raises(SessionError) as exc_info:
        await client.refresh(refresh_req)
    assert exc_info.value.code == SessionError.Code.REVOKED


async def test_list_user_sessions() -> None:
    client = InMemorySessionClient()
    await client.create(CreateSessionRequest(user_id="user-1", ttl_seconds=3600))
    await client.create(CreateSessionRequest(user_id="user-1", ttl_seconds=3600))
    await client.create(CreateSessionRequest(user_id="user-2", ttl_seconds=3600))

    sessions = await client.list_user_sessions("user-1")
    assert len(sessions) == 2


async def test_revoke_all() -> None:
    client = InMemorySessionClient()
    await client.create(CreateSessionRequest(user_id="user-1", ttl_seconds=3600))
    await client.create(CreateSessionRequest(user_id="user-1", ttl_seconds=3600))

    count = await client.revoke_all("user-1")
    assert count == 2

    sessions = await client.list_user_sessions("user-1")
    assert all(s.revoked for s in sessions)


async def test_create_with_metadata() -> None:
    client = InMemorySessionClient()
    req = CreateSessionRequest(
        user_id="user-1", ttl_seconds=3600, metadata={"device": "mobile"}
    )
    session = await client.create(req)
    assert session.metadata["device"] == "mobile"


async def test_revoke_not_found() -> None:
    client = InMemorySessionClient()
    with pytest.raises(SessionError) as exc_info:
        await client.revoke("nonexistent")
    assert exc_info.value.code == SessionError.Code.NOT_FOUND


async def test_revoke_all_returns_zero_for_unknown_user() -> None:
    client = InMemorySessionClient()
    count = await client.revoke_all("unknown")
    assert count == 0
