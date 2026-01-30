"""Fluent API for building policy rules."""

from __future__ import annotations

from k1s0_auth.policy.models import Action, PolicyRule


class PolicyBuilder:
    """Fluent builder for constructing lists of :class:`PolicyRule`.

    Example::

        rules = (
            PolicyBuilder()
            .allow_admin("orders.*")
            .allow_read("orders.*", roles=["viewer"])
            .allow_write("orders.*", roles=["editor"])
            .build()
        )
    """

    def __init__(self) -> None:
        self._rules: list[PolicyRule] = []

    def allow_admin(self, resource: str) -> PolicyBuilder:
        """Allow ADMIN action for the ``admin`` role on the given resource.

        Args:
            resource: Resource pattern.
        """
        self._rules.append(
            PolicyRule(
                action=Action.ADMIN,
                resource_pattern=resource,
                required_roles=["admin"],
                allow=True,
            )
        )
        return self

    def allow_read(self, resource: str, roles: list[str] | None = None) -> PolicyBuilder:
        """Allow READ action for the given roles.

        Args:
            resource: Resource pattern.
            roles: Required roles. Defaults to ``["viewer"]``.
        """
        self._rules.append(
            PolicyRule(
                action=Action.READ,
                resource_pattern=resource,
                required_roles=roles or ["viewer"],
                allow=True,
            )
        )
        return self

    def allow_write(self, resource: str, roles: list[str] | None = None) -> PolicyBuilder:
        """Allow WRITE action for the given roles.

        Args:
            resource: Resource pattern.
            roles: Required roles. Defaults to ``["editor"]``.
        """
        self._rules.append(
            PolicyRule(
                action=Action.WRITE,
                resource_pattern=resource,
                required_roles=roles or ["editor"],
                allow=True,
            )
        )
        return self

    def custom(self, rule: PolicyRule) -> PolicyBuilder:
        """Add a custom policy rule.

        Args:
            rule: The rule to add.
        """
        self._rules.append(rule)
        return self

    def build(self) -> list[PolicyRule]:
        """Return the accumulated list of policy rules."""
        return list(self._rules)
