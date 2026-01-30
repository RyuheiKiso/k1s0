"""Policy repository interfaces and in-memory implementation."""

from __future__ import annotations

from abc import ABC, abstractmethod

from k1s0_auth.policy.models import PolicyRule


class PolicyRepository(ABC):
    """Abstract repository for loading policy rules."""

    @abstractmethod
    async def get_rules(self, resource: str) -> list[PolicyRule]:
        """Return all rules applicable to the given resource.

        Args:
            resource: The resource identifier.

        Returns:
            List of matching policy rules.
        """


class InMemoryPolicyRepository(PolicyRepository):
    """In-memory policy repository for testing and simple deployments."""

    def __init__(self) -> None:
        self._rules: dict[str, list[PolicyRule]] = {}

    def add_rule(self, resource: str, rule: PolicyRule) -> None:
        """Add a rule for a resource pattern.

        Args:
            resource: The resource key to store the rule under.
            rule: The policy rule to add.
        """
        self._rules.setdefault(resource, []).append(rule)

    async def get_rules(self, resource: str) -> list[PolicyRule]:
        """Return rules matching the resource exactly or by pattern."""
        results: list[PolicyRule] = []
        for pattern, rules in self._rules.items():
            if self._matches(pattern, resource):
                results.extend(rules)
        return results

    @staticmethod
    def _matches(pattern: str, resource: str) -> bool:
        """Simple glob matching: ``*`` matches any segment."""
        if pattern == resource:
            return True
        if pattern.endswith(".*"):
            prefix = pattern[:-2]
            return resource == prefix or resource.startswith(prefix + ".")
        return False
