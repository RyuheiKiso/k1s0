"""HTTP health check implementation."""

from __future__ import annotations

from urllib.error import URLError
from urllib.request import Request, urlopen

from .checker import HealthCheck


class HttpHealthCheck(HealthCheck):
    """HTTP GET リクエストでヘルスを確認する HealthCheck 実装。

    stdlib の urllib を使用し、外部依存なし。
    """

    def __init__(
        self,
        url: str,
        *,
        timeout_seconds: float = 5.0,
        name: str | None = None,
    ) -> None:
        self._name = name or "http"
        self._url = url
        self._timeout = timeout_seconds

    @property
    def name(self) -> str:
        return self._name

    async def check(self) -> None:
        """HTTP GET を送信し、2xx でなければ例外を送出する。

        Note: urllib はブロッキング I/O だが、ヘルスチェックの HTTP コールは
        軽量かつ短時間であり、タイムアウトも設定されているため実用上問題ない。
        本番で非同期 HTTP が必要な場合は httpx 等に差し替え可能。
        """
        try:
            req = Request(self._url, method="GET")
            with urlopen(req, timeout=self._timeout) as resp:
                status = resp.status
                if status < 200 or status >= 300:
                    raise RuntimeError(
                        f"HTTP {self._url} returned status {status}"
                    )
        except URLError as exc:
            raise RuntimeError(f"HTTP check failed: {exc}") from exc
        except TimeoutError as exc:
            raise RuntimeError(
                f"HTTP check timeout: {self._url}"
            ) from exc
