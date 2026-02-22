"""RbacChecker のユニットテスト"""

from k1s0_auth.models import TokenClaims
from k1s0_auth.rbac import RbacChecker


def make_claims(roles: list[str]) -> TokenClaims:
    return TokenClaims(
        sub="user1", iss="https://iss", aud=["api"], exp=9999999999, iat=0, roles=roles
    )


def test_check_permission_granted() -> None:
    """ロールに対応するパーミッションが許可されること。"""
    checker = RbacChecker({"admin": ["user:read", "user:write"]})
    claims = make_claims(["admin"])
    assert checker.check_permission(claims, "user", "read") is True
    assert checker.check_permission(claims, "user", "write") is True


def test_check_permission_denied() -> None:
    """ロールに対応するパーミッションがない場合は拒否されること。"""
    checker = RbacChecker({"viewer": ["user:read"]})
    claims = make_claims(["viewer"])
    assert checker.check_permission(claims, "user", "write") is False


def test_check_permission_wildcard_action() -> None:
    """resource:* で全アクションが許可されること。"""
    checker = RbacChecker({"admin": ["user:*"]})
    claims = make_claims(["admin"])
    assert checker.check_permission(claims, "user", "delete") is True


def test_check_permission_wildcard_all() -> None:
    """*:* で全リソース・アクションが許可されること。"""
    checker = RbacChecker({"superadmin": ["*:*"]})
    claims = make_claims(["superadmin"])
    assert checker.check_permission(claims, "any", "thing") is True


def test_check_permission_no_roles() -> None:
    """ロールなしの場合は拒否されること。"""
    checker = RbacChecker({"admin": ["user:read"]})
    claims = make_claims([])
    assert checker.check_permission(claims, "user", "read") is False


def test_require_role_true() -> None:
    """ロールを持つ場合は True を返すこと。"""
    checker = RbacChecker()
    claims = make_claims(["admin", "viewer"])
    assert checker.require_role(claims, "admin") is True


def test_require_role_false() -> None:
    """ロールを持たない場合は False を返すこと。"""
    checker = RbacChecker()
    claims = make_claims(["viewer"])
    assert checker.require_role(claims, "admin") is False
