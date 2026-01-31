"""Policy evaluation engine."""

from __future__ import annotations

from abc import ABC, abstractmethod

from k1s0_auth.policy.models import PolicyRequest, PolicyRule
from k1s0_auth.policy.repository import PolicyRepository


class PolicyEvaluator(ABC):
    """Abstract policy evaluator."""

    @abstractmethod
    async def evaluate(self, request: PolicyRequest) -> bool:
        """Evaluate whether the request should be allowed.

        Args:
            request: The authorization request.

        Returns:
            ``True`` if access is granted, ``False`` otherwise.
        """


class RepositoryPolicyEvaluator(PolicyEvaluator):
    """Evaluates authorization using rules from a :class:`PolicyRepository`.

    Default-deny: if no matching rule is found, access is denied.

    Args:
        repository: The policy repository to load rules from.
    """

    def __init__(self, repository: PolicyRepository) -> None:
        self._repository = repository

    async def evaluate(self, request: PolicyRequest) -> bool:
        """Evaluate the request against stored policy rules."""
        rules = await self._repository.get_rules(request.resource)
        matching = [r for r in rules if r.action == request.action]

        if not matching:
            return False

        for rule in matching:
            if self._rule_matches(rule, request):
                return rule.allow

        return False

    @staticmethod
    def _rule_matches(rule: PolicyRule, request: PolicyRequest) -> bool:
        """Check whether the subject satisfies the rule's requirements."""
        subject = request.subject

        if rule.required_roles and not (set(subject.roles) & set(rule.required_roles)):
            return False

        if rule.required_permissions and not (
            set(subject.permissions) & set(rule.required_permissions)
        ):
            return False

        return True
