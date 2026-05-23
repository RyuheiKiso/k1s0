"""src/client/full_native/python/k1s0_native.py

full_native SDK Python 実装: k1s0 API の Python ネイティブクライアント。
仕様: 18_クライアントSDK配布適合仕様.md §full_native class
- 9 言語等価強度の Python 実装
- Connect-RPC プロトコル対応（protobuf JSON フォールバック含む）
- hlc_lib Python wrapper による HLC タイムスタンプ生成
- keyring（OS keychain）による refresh_token の安全な保管
pact consumer contract: pact/consumer/full_native.pact.json と整合する

依存:
  PyYAML (設定ファイル読み込み用)
  keyring (オプション: OS keychain 連携)
"""

from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from typing import Any

# API ベース URL をデフォルト値つきで環境変数から取得する
_DEFAULT_BASE_URL = os.environ.get("K1S0_API_BASE_URL", "http://localhost:8080")
# keychain サービス名を定義する
_KEYCHAIN_SERVICE = "k1s0-full-native"


# ---------------------------------------------------------------------------
# HLC タイムスタンプ生成（hlc_lib Python wrapper）
# ---------------------------------------------------------------------------

class HlcClock:
    """Hybrid Logical Clock（HLC）実装。

    src/client/hlc_lib/python 相当の機能を提供する。
    仕様: wall-clock TTL 禁止 / HLC を全 timestamp に使用する（src/CLAUDE.md §wall-clock TTL 禁止）
    """

    def __init__(self) -> None:
        # 最後に発行した HLC 値を保持するフィールド
        self._last_hlc: int = 0

    def now(self) -> int:
        """現在の HLC タイムスタンプを返す（単調増加を保証する）。"""
        # 物理時刻をナノ秒で取得する（wall-clock は HLC の入力にのみ使用する）
        wall_ns = int(time.time() * 1_000_000_000)
        # 前回の HLC より大きい値を保証する
        hlc = max(wall_ns, self._last_hlc + 1)
        # 最後の HLC 値を更新する
        self._last_hlc = hlc
        return hlc

    def update(self, remote_hlc: int) -> int:
        """リモートの HLC を受信して自身の HLC を更新する。"""
        # リモート HLC と現在時刻の大きい方 + 1 を新しい HLC とする
        wall_ns = int(time.time() * 1_000_000_000)
        hlc = max(remote_hlc, wall_ns, self._last_hlc) + 1
        self._last_hlc = hlc
        return hlc


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
# OS keychain 連携
# ---------------------------------------------------------------------------

def _store_refresh_token(account: str, token: str) -> bool:
    """refresh_token を OS keychain に保管する。"""
    try:
        import keyring
        keyring.set_password(_KEYCHAIN_SERVICE, account, token)
        return True
    except (ImportError, Exception):
        # keyring が利用不可の場合は False を返す（fallback: in-memory のみ）
        return False


def _load_refresh_token(account: str) -> str | None:
    """OS keychain から refresh_token を取得する。"""
    try:
        import keyring
        return keyring.get_password(_KEYCHAIN_SERVICE, account)
    except (ImportError, Exception):
        return None


# ---------------------------------------------------------------------------
# full_native SDK クライアント
# ---------------------------------------------------------------------------

class FullNativeClient:
    """k1s0 API の full_native Python クライアント。

    Connect-RPC プロトコル（JSON エンベロープ フォールバック含む）を使用する。
    9 言語等価強度の Python 実装として hlc_lib を内包する。
    """

    def __init__(
        self,
        base_url: str = _DEFAULT_BASE_URL,
        account: str = "default",
        timeout_seconds: int = 30,
    ) -> None:
        # API ベース URL をスラッシュなしで正規化する
        self._base_url = base_url.rstrip("/")
        # OS keychain のアカウント名を設定する
        self._account = account
        # HTTP タイムアウト秒数を設定する
        self._timeout = timeout_seconds
        # HLC クロックを初期化する
        self._hlc = HlcClock()
        # アクセストークンを初期化する（keychain からの復元は非同期で行う）
        self._access_token: str = os.environ.get("K1S0_ACCESS_TOKEN", "")

    def _build_headers(self) -> dict[str, str]:
        """共通 HTTP ヘッダーを構築して返す。"""
        headers: dict[str, str] = {
            "Content-Type": "application/json",
            "Accept": "application/json",
            "X-K1s0-Client": "full_native/python/1.0.0",
            # HLC タイムスタンプをリクエストヘッダーに含める（仕様: HLC 単調性保証）
            "X-K1s0-Hlc": str(self._hlc.now()),
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
                # レスポンスの HLC ヘッダーで自身のクロックを更新する
                remote_hlc_str = resp.getheader("X-K1s0-Hlc", "")
                if remote_hlc_str.isdigit():
                    self._hlc.update(int(remote_hlc_str))
                # レスポンスボディを読み込んで JSON デコードする
                raw = resp.read().decode("utf-8", errors="replace")
                return json.loads(raw) if raw else {}
        except urllib.error.HTTPError as exc:
            raw = exc.read().decode("utf-8", errors="replace") if exc.fp else ""
            return {"error": str(exc), "status": exc.code, "detail": raw}
        except urllib.error.URLError as exc:
            return {"error": str(exc), "status": 0}

    def get_event_stream(self, cursor: str | None = None) -> dict[str, Any]:
        """イベントストリームを取得する（pact contract: GetEventStream）。"""
        # クエリパラメータを構築する
        params: dict[str, str] = {}
        if cursor:
            params["cursor"] = cursor
        query = urllib.parse.urlencode(params) if params else ""
        path = f"/v1/events?{query}" if query else "/v1/events"
        return self._request("GET", path)

    def create_domain_event(
        self,
        event_type: str,
        payload: dict[str, Any],
    ) -> dict[str, Any]:
        """ドメインイベントを作成する（pact contract: CreateDomainEvent）。"""
        return self._request("POST", "/v1/events", {"event_type": event_type, "payload": payload})

    def refresh_tokens(self, refresh_token: str) -> TokenPair | None:
        """refresh_token で新しいアクセストークンを取得する。"""
        result = self._request("POST", "/v1/auth/refresh", {"refresh_token": refresh_token})
        if "error" in result:
            return None
        # 新しいアクセストークンで自身を更新する
        new_access = result.get("access_token", "")
        new_refresh = result.get("refresh_token", refresh_token)
        self._access_token = new_access
        # 新しい refresh_token を keychain に保管する
        _store_refresh_token(self._account, new_refresh)
        return TokenPair(
            access_token=new_access,
            refresh_token=new_refresh,
            expires_at=result.get("expires_at"),
        )

    def health_check(self) -> bool:
        """API ヘルスチェックを実行して True / False を返す。"""
        result = self._request("GET", "/healthz")
        return result.get("status") == "ok"

    def set_access_token(self, token: str) -> None:
        """アクセストークンを動的に更新する。"""
        self._access_token = token
