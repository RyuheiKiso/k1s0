"""認証関連データモデル"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class TokenClaims:
    """JWT トークンクレーム。"""

    sub: str
    iss: str
    aud: list[str]
    exp: int
    iat: int
    scope: str = ""
    roles: list[str] = field(default_factory=list)
    extra: dict[str, Any] = field(default_factory=dict)

    def has_scope(self, scope: str) -> bool:
        """指定されたスコープを持つか確認する。"""
        return scope in self.scope.split()

    def has_role(self, role: str) -> bool:
        """指定されたロールを持つか確認する。"""
        return role in self.roles


@dataclass
class DeviceFlowResponse:
    """デバイスフロー開始レスポンス。"""

    device_code: str
    user_code: str
    verification_uri: str
    expires_in: int
    interval: int


@dataclass
class TokenResponse:
    """トークン取得レスポンス。"""

    access_token: str
    token_type: str
    expires_in: int
    scope: str = ""
    refresh_token: str = ""
