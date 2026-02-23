"""テスト用 JWT トークン生成ヘルパー。"""

from __future__ import annotations

import base64
import json
import time
from dataclasses import dataclass, field


@dataclass
class TestClaims:
    """テスト用 JWT クレーム。"""

    sub: str
    roles: list[str] = field(default_factory=list)
    tenant_id: str | None = None
    iat: int | None = None
    exp: int | None = None

    def to_dict(self) -> dict:
        now = int(time.time())
        d: dict = {
            "sub": self.sub,
            "roles": self.roles,
            "iat": self.iat or now,
            "exp": self.exp or now + 3600,
        }
        if self.tenant_id is not None:
            d["tenant_id"] = self.tenant_id
        return d

    @classmethod
    def from_dict(cls, data: dict) -> TestClaims:
        return cls(
            sub=data.get("sub", ""),
            roles=data.get("roles", []),
            tenant_id=data.get("tenant_id"),
            iat=data.get("iat"),
            exp=data.get("exp"),
        )


class JwtTestHelper:
    """テスト用 JWT トークン生成ヘルパー (HS256 簡易実装)。"""

    def __init__(self, secret: str) -> None:
        self._secret = secret

    def create_admin_token(self) -> str:
        """管理者トークンを生成する。"""
        return self.create_token(TestClaims(sub="admin", roles=["admin"]))

    def create_user_token(self, user_id: str, roles: list[str]) -> str:
        """ユーザートークンを生成する。"""
        return self.create_token(TestClaims(sub=user_id, roles=roles))

    def create_token(self, claims: TestClaims) -> str:
        """カスタムクレームでトークンを生成する。"""
        header = _base64url_encode('{"alg":"HS256","typ":"JWT"}')
        payload = _base64url_encode(json.dumps(claims.to_dict(), separators=(",", ":")))
        signing_input = f"{header}.{payload}"
        signature = _base64url_encode(f"{signing_input}:{self._secret}")
        return f"{signing_input}.{signature}"

    def decode_claims(self, token: str) -> TestClaims | None:
        """トークンのペイロードをデコードしてクレームを返す。"""
        parts = token.split(".")
        if len(parts) != 3:
            return None
        try:
            payload_json = _base64url_decode(parts[1])
            data = json.loads(payload_json)
            return TestClaims.from_dict(data)
        except Exception:
            return None


def _base64url_encode(data: str) -> str:
    return base64.urlsafe_b64encode(data.encode()).rstrip(b"=").decode()


def _base64url_decode(data: str) -> str:
    padded = data + "=" * (4 - len(data) % 4) if len(data) % 4 else data
    return base64.urlsafe_b64decode(padded).decode()
