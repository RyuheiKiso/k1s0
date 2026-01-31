"""Tests for Claims model."""

from __future__ import annotations

import pytest

from k1s0_auth.jwt.claims import Claims


class TestClaims:
    """Test suite for the Claims model."""

    def test_has_role_returns_true(self) -> None:
        claims = Claims(sub="u1", roles=["admin", "viewer"])
        assert claims.has_role("admin") is True

    def test_has_role_returns_false(self) -> None:
        claims = Claims(sub="u1", roles=["viewer"])
        assert claims.has_role("admin") is False

    def test_has_permission_returns_true(self) -> None:
        claims = Claims(sub="u1", permissions=["orders.read"])
        assert claims.has_permission("orders.read") is True

    def test_has_permission_returns_false(self) -> None:
        claims = Claims(sub="u1")
        assert claims.has_permission("orders.read") is False

    def test_has_any_role_returns_true(self) -> None:
        claims = Claims(sub="u1", roles=["editor"])
        assert claims.has_any_role(["admin", "editor"]) is True

    def test_has_any_role_returns_false(self) -> None:
        claims = Claims(sub="u1", roles=["viewer"])
        assert claims.has_any_role(["admin", "editor"]) is False

    def test_defaults(self) -> None:
        claims = Claims(sub="u1")
        assert claims.roles == []
        assert claims.permissions == []
        assert claims.groups == []
        assert claims.tenant_id is None
        assert claims.custom == {}
