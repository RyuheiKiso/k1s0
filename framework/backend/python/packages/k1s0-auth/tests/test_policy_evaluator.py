"""Tests for PolicyEvaluator."""

from __future__ import annotations

import pytest

from k1s0_auth.policy.evaluator import RepositoryPolicyEvaluator
from k1s0_auth.policy.models import Action, PolicyRequest, PolicyRule, PolicySubject
from k1s0_auth.policy.repository import InMemoryPolicyRepository


@pytest.fixture()
def repo() -> InMemoryPolicyRepository:
    r = InMemoryPolicyRepository()
    r.add_rule(
        "orders.*",
        PolicyRule(action=Action.ADMIN, resource_pattern="orders.*", required_roles=["admin"]),
    )
    r.add_rule(
        "orders.*",
        PolicyRule(action=Action.READ, resource_pattern="orders.*", required_roles=["viewer"]),
    )
    r.add_rule(
        "orders.*",
        PolicyRule(action=Action.WRITE, resource_pattern="orders.*", required_roles=["editor"]),
    )
    return r


class TestRepositoryPolicyEvaluator:
    """Test suite for RepositoryPolicyEvaluator."""

    @pytest.mark.asyncio()
    async def test_admin_allowed(self, repo: InMemoryPolicyRepository) -> None:
        evaluator = RepositoryPolicyEvaluator(repo)
        request = PolicyRequest(
            subject=PolicySubject(sub="u1", roles=["admin"]),
            action=Action.ADMIN,
            resource="orders.list",
        )
        assert await evaluator.evaluate(request) is True

    @pytest.mark.asyncio()
    async def test_read_allowed(self, repo: InMemoryPolicyRepository) -> None:
        evaluator = RepositoryPolicyEvaluator(repo)
        request = PolicyRequest(
            subject=PolicySubject(sub="u1", roles=["viewer"]),
            action=Action.READ,
            resource="orders.list",
        )
        assert await evaluator.evaluate(request) is True

    @pytest.mark.asyncio()
    async def test_undefined_resource_denied(self, repo: InMemoryPolicyRepository) -> None:
        evaluator = RepositoryPolicyEvaluator(repo)
        request = PolicyRequest(
            subject=PolicySubject(sub="u1", roles=["viewer"]),
            action=Action.READ,
            resource="invoices.list",
        )
        assert await evaluator.evaluate(request) is False

    @pytest.mark.asyncio()
    async def test_wrong_role_denied(self, repo: InMemoryPolicyRepository) -> None:
        evaluator = RepositoryPolicyEvaluator(repo)
        request = PolicyRequest(
            subject=PolicySubject(sub="u1", roles=["viewer"]),
            action=Action.WRITE,
            resource="orders.create",
        )
        assert await evaluator.evaluate(request) is False
