"""RBAC チェッカー"""

from __future__ import annotations

from .models import TokenClaims


class RbacChecker:
    """ロールベースアクセス制御チェッカー。"""

    def __init__(self, permission_map: dict[str, list[str]] | None = None) -> None:
        """
        Args:
            permission_map: {role: [resource:action, ...]} の辞書。
                           例: {"admin": ["user:read", "user:write"], "viewer": ["user:read"]}
        """
        self._permission_map = permission_map or {}

    def check_permission(self, claims: TokenClaims, resource: str, action: str) -> bool:
        """クレームが指定リソースとアクションへのアクセス権を持つか確認する。"""
        permission = f"{resource}:{action}"
        for role in claims.roles:
            allowed = self._permission_map.get(role, [])
            if permission in allowed or f"{resource}:*" in allowed or "*:*" in allowed:
                return True
        return False

    def require_role(self, claims: TokenClaims, role: str) -> bool:
        """クレームが指定ロールを持つか確認する。"""
        return role in claims.roles
