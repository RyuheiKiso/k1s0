"""Tests for refresh token management."""

from __future__ import annotations

import pytest

from k1s0_auth.refresh import InMemoryRefreshTokenStore, RefreshTokenManager


@pytest.fixture()
def manager() -> RefreshTokenManager:
    store = InMemoryRefreshTokenStore()
    return RefreshTokenManager(store, secret_key="test-secret-key", token_ttl=3600)


class TestRefreshTokenManager:
    """Test suite for RefreshTokenManager."""

    @pytest.mark.asyncio()
    async def test_issue_and_verify(self, manager: RefreshTokenManager) -> None:
        token = await manager.issue("user-1")
        sub = await manager.verify(token)
        assert sub == "user-1"

    @pytest.mark.asyncio()
    async def test_revoke(self, manager: RefreshTokenManager) -> None:
        token = await manager.issue("user-1")
        await manager.revoke(token)
        with pytest.raises(ValueError, match="revoked"):
            await manager.verify(token)

    @pytest.mark.asyncio()
    async def test_different_subjects(self, manager: RefreshTokenManager) -> None:
        t1 = await manager.issue("user-1")
        t2 = await manager.issue("user-2")
        assert await manager.verify(t1) == "user-1"
        assert await manager.verify(t2) == "user-2"
