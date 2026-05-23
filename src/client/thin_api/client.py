"""src/client/thin_api/client.py

thin_api SDK: k1s0 API への最小限の HTTP クライアント。
仕様: 18_クライアントSDK配布適合仕様.md §thin_api class
- Connect-RPC (HTTP/2 + protobuf) ベースの API コール
- access_token / refresh_token の自動管理
- pact consumer contract: pact/consumer/thin_api.pact.json と整合する

環境変数:
  K1S0_API_BASE_URL: API エンドポイント（デフォルト: http://localhost:8080）
  K1S0_ACCESS_TOKEN: アクセストークン（テスト用）
"""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from typing import Any

# API ベース URL をデフォルト値つきで環境変数から取得する
_DEFAULT_BASE_URL = os.environ.get("K1S0_API_BASE_URL", "http://localhost:8080")


# ---------------------------------------------------------------------------
# 認証情報データクラス
# ---------------------------------------------------------------------------

@dataclass
class TokenPair:
    """access_token / refresh_token のペアを保持するデータクラス。"""
    # アクセストークン（短期: 15 分）
    access_token: str
    # リフレッシュトークン（長期: 30 日）
    refresh_token: str
    # トークンの有効期限（UNIX time, None は無期限）
    expires_at: float | None = None


# ---------------------------------------------------------------------------
# thin_api SDK クライアント
# ---------------------------------------------------------------------------

class ThinApiClient:
    """k1s0 API の thin HTTP クライアント。

    Connect-RPC プロトコル（Content-Type: application/connect+proto）を使用する。
    pact/consumer/thin_api.pact.json で定義された contract に準拠する。
    """

    def __init__(
        self,
        base_url: str = _DEFAULT_BASE_URL,
        access_token: str | None = None,
        timeout_seconds: int = 30,
    ) -> None:
        # API ベース URL をスラッシュなしで正規化する
        self._base_url = base_url.rstrip("/")
        # アクセストークン（None の場合は未認証）
        self._access_token = access_token or os.environ.get("K1S0_ACCESS_TOKEN", "")
        # HTTP タイムアウト秒数を設定する
        self._timeout = timeout_seconds

    def _build_headers(self) -> dict[str, str]:
        """共通 HTTP ヘッダーを構築して返す。"""
        headers: dict[str, str] = {
            "Content-Type": "application/json",
            "Accept": "application/json",
            "X-K1s0-Client": "thin_api/python/1.0.0",
        }
        # アクセストークンが設定されている場合は Authorization ヘッダーを追加する
        if self._access_token:
            headers["Authorization"] = f"Bearer {self._access_token}"
        return headers

    def _request(
        self,
        method: str,
        path: str,
        body: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """HTTP リクエストを実行して JSON レスポンスを返す。"""
        url = f"{self._base_url}{path}"
        data = json.dumps(body).encode("utf-8") if body else None
        req = urllib.request.Request(
            url,
            data=data,
            headers=self._build_headers(),
            method=method,
        )
        try:
            with urllib.request.urlopen(req, timeout=self._timeout) as resp:
                # レスポンスボディを読み込んで JSON デコードする
                raw = resp.read().decode("utf-8", errors="replace")
                return json.loads(raw) if raw else {}
        except urllib.error.HTTPError as exc:
            # HTTP エラーのレスポンスボディをエラーとして返す
            raw = exc.read().decode("utf-8", errors="replace") if exc.fp else ""
            return {"error": str(exc), "status": exc.code, "detail": raw}
        except urllib.error.URLError as exc:
            # ネットワークエラーを dict で返す
            return {"error": str(exc), "status": 0}

    def get_event_stream(self, tenant_id: str, cursor: str | None = None) -> dict[str, Any]:
        """イベントストリームを取得する（pact contract: GetEventStream）。"""
        # クエリパラメータを構築する
        params: dict[str, str] = {}
        if cursor:
            params["cursor"] = cursor
        # パスにクエリパラメータを追加する
        query = urllib.parse.urlencode(params) if params else ""
        path = f"/v1/events?{query}" if query else "/v1/events"
        return self._request("GET", path)

    def create_domain_event(
        self, event_type: str, payload: dict[str, Any]
    ) -> dict[str, Any]:
        """ドメインイベントを作成する（pact contract: CreateDomainEvent）。"""
        return self._request("POST", "/v1/events", {"event_type": event_type, "payload": payload})

    def health_check(self) -> bool:
        """API ヘルスチェックを実行して True / False を返す。"""
        result = self._request("GET", "/healthz")
        return result.get("status") == "ok"

    def set_access_token(self, token: str) -> None:
        """アクセストークンを動的に更新する（refresh_token ローテーション後に呼ぶ）。"""
        self._access_token = token
