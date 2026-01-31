"""Tests for token blacklist."""

from __future__ import annotations

import time

import pytest

from k1s0_auth.blacklist import InMemoryBlacklist


class TestInMemoryBlacklist:
    """Test suite for InMemoryBlacklist."""

    @pytest.mark.asyncio()
    async def test_add_and_check(self) -> None:
        bl = InMemoryBlacklist()
        await bl.add("jti-1", time.time() + 3600)
        assert await bl.is_blacklisted("jti-1") is True

    @pytest.mark.asyncio()
    async def test_not_blacklisted(self) -> None:
        bl = InMemoryBlacklist()
        assert await bl.is_blacklisted("jti-unknown") is False

    @pytest.mark.asyncio()
    async def test_expired_entry_cleaned_up(self) -> None:
        bl = InMemoryBlacklist()
        await bl.add("jti-old", time.time() - 1)
        assert await bl.is_blacklisted("jti-old") is False
