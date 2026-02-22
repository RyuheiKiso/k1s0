"""service_auth データモデル"""

from __future__ import annotations

import re
import time
from dataclasses import dataclass, field
from typing import Any


@dataclass
class ServiceToken:
    """サービストークン。"""

    access_token: str
    token_type: str
    expires_at: float  # Unix timestamp
    scope: str = ""

    def is_expired(self, buffer_seconds: float = 30.0) -> bool:
        """トークンの有効期限が切れているか確認する（バッファ付き）。"""
        return time.time() >= (self.expires_at - buffer_seconds)

    @classmethod
    def from_response(cls, response: dict[str, Any]) -> ServiceToken:
        """OAuth2 レスポンス辞書から ServiceToken を生成する。"""
        expires_in = int(response.get("expires_in", 3600))
        return cls(
            access_token=response["access_token"],
            token_type=response.get("token_type", "Bearer"),
            expires_at=time.time() + expires_in,
            scope=response.get("scope", ""),
        )


@dataclass
class ServiceClaims:
    """サービス JWT クレーム。"""

    sub: str
    iss: str
    scope: str = ""
    extra: dict[str, Any] = field(default_factory=dict)


@dataclass
class SpiffeId:
    """SPIFFE ID（spiffe://<trust-domain>/ns/<namespace>/sa/<service-account>）。"""

    trust_domain: str
    namespace: str
    service_account: str

    _PATTERN = re.compile(
        r"^spiffe://(?P<trust_domain>[^/]+)/ns/(?P<namespace>[^/]+)/sa/(?P<service_account>[^/]+)$"
    )

    @classmethod
    def parse(cls, uri: str) -> SpiffeId:
        """SPIFFE URI をパースする。

        Raises:
            ValueError: URI の形式が不正な場合
        """
        match = cls._PATTERN.match(uri)
        if not match:
            raise ValueError(
                f"Invalid SPIFFE ID format: {uri!r}. "
                "Expected: spiffe://<trust-domain>/ns/<namespace>/sa/<service-account>"
            )
        return cls(
            trust_domain=match.group("trust_domain"),
            namespace=match.group("namespace"),
            service_account=match.group("service_account"),
        )

    def to_uri(self) -> str:
        """SPIFFE URI 文字列に変換する。"""
        return f"spiffe://{self.trust_domain}/ns/{self.namespace}/sa/{self.service_account}"


@dataclass
class ServiceAuthConfig:
    """service_auth 設定。"""

    token_url: str
    client_id: str
    client_secret: str
    scope: str = ""
    jwks_uri: str = ""
    timeout_seconds: float = 10.0
