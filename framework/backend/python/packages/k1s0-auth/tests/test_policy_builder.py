"""Tests for PolicyBuilder."""

from __future__ import annotations

from k1s0_auth.policy.builder import PolicyBuilder
from k1s0_auth.policy.models import Action


class TestPolicyBuilder:
    """Test suite for PolicyBuilder."""

    def test_allow_admin(self) -> None:
        rules = PolicyBuilder().allow_admin("orders.*").build()
        assert len(rules) == 1
        assert rules[0].action == Action.ADMIN
        assert rules[0].required_roles == ["admin"]
        assert rules[0].allow is True

    def test_allow_read_default_roles(self) -> None:
        rules = PolicyBuilder().allow_read("orders.*").build()
        assert rules[0].required_roles == ["viewer"]

    def test_allow_write_custom_roles(self) -> None:
        rules = PolicyBuilder().allow_write("orders.*", roles=["manager"]).build()
        assert rules[0].required_roles == ["manager"]

    def test_chaining(self) -> None:
        rules = (
            PolicyBuilder()
            .allow_admin("orders.*")
            .allow_read("orders.*")
            .allow_write("orders.*")
            .build()
        )
        assert len(rules) == 3

    def test_custom_rule(self) -> None:
        from k1s0_auth.policy.models import PolicyRule

        custom = PolicyRule(
            action=Action.DELETE,
            resource_pattern="orders.*",
            required_permissions=["orders.delete"],
        )
        rules = PolicyBuilder().custom(custom).build()
        assert len(rules) == 1
        assert rules[0].action == Action.DELETE
