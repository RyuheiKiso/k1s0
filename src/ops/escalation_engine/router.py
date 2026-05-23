"""src/ops/escalation_engine/router.py

マルチプロバイダー通知ルーター。
仕様: 17_運用ループ適合仕様.md §self_escalation_engine

- NotificationProvider: 通知プロバイダーの抽象基底クラス
- MattermostProvider: Incoming Webhook を使った Mattermost 通知実装
- SlackProvider: Incoming Webhook を使った Slack 通知実装
- PagerDutyProvider: REST API v2 を使った PagerDuty 通知実装
- ProviderRouter: signal_class と severity に基づいてプロバイダーにルーティングする
- 重複排除: 同一 alert_fingerprint は 5 分以内に再通知しない
- フェイルオーバー: プライマリプロバイダー失敗時にセカンダリへ切り替える
"""

from __future__ import annotations

import hashlib
import json
import logging
import time
import urllib.error
import urllib.parse
import urllib.request
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any

# モジュールロガーを初期化する
logger = logging.getLogger(__name__)

# 重複排除ウィンドウ（秒）: この時間内の同一 fingerprint は再通知しない
_DEDUP_WINDOW_S = 300


# ---------------------------------------------------------------------------
# NotificationPayload データクラス
# ---------------------------------------------------------------------------

@dataclass
class NotificationPayload:
    """プロバイダーに渡す通知ペイロードの共通構造。"""

    # アラート識別子（Alertmanager の alertname）
    alert_name: str
    # 通知の表題
    title: str
    # 通知の本文
    body: str
    # signal_class 名（v1_page / v1_freeze / v1_notify / v1_audit / v1_slo_breach）
    signal_class: str
    # 重要度（critical / warning / info）
    severity: str
    # インシデント ID
    incident_id: str
    # アラートフィンガープリント（重複排除に使用する）
    alert_fingerprint: str
    # HLC タイムスタンプ（"wall_ms:logical" 形式）
    hlc_ts: str
    # ルートコーズとなる runbook URL
    runbook_url: str = ""
    # 追加メタデータ
    labels: dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """NotificationPayload を辞書形式にシリアライズする。"""
        return {
            "alert_name": self.alert_name,
            "title": self.title,
            "body": self.body,
            "signal_class": self.signal_class,
            "severity": self.severity,
            "incident_id": self.incident_id,
            "alert_fingerprint": self.alert_fingerprint,
            "hlc_ts": self.hlc_ts,
            "runbook_url": self.runbook_url,
            "labels": self.labels,
        }


# ---------------------------------------------------------------------------
# NotificationResult データクラス
# ---------------------------------------------------------------------------

@dataclass
class NotificationResult:
    """通知送信の結果を保持するデータクラス。"""

    # 送信成功フラグ
    success: bool
    # プロバイダー名
    provider_name: str
    # HTTP ステータスコード（HTTP を使わないプロバイダーでは 0）
    status_code: int = 0
    # エラーメッセージ（成功の場合は空文字列）
    error_message: str = ""
    # dry-run モードフラグ
    dry_run: bool = False
    # 送信時刻（Unix タイムスタンプ）
    sent_at: float = field(default_factory=time.time)

    def __post_init__(self) -> None:
        """sent_at のデフォルト値を設定する。"""
        if self.sent_at == 0.0:
            self.sent_at = time.time()


# ---------------------------------------------------------------------------
# NotificationProvider 抽象基底クラス
# ---------------------------------------------------------------------------

class NotificationProvider(ABC):
    """通知プロバイダーの抽象基底クラス。全プロバイダーはこれを継承する。"""

    @property
    @abstractmethod
    def name(self) -> str:
        """プロバイダー名を返す（ログ識別用）。"""
        ...

    @abstractmethod
    def send(self, payload: NotificationPayload, dry_run: bool = False) -> NotificationResult:
        """通知を送信する。

        Args:
            payload: 送信する通知ペイロード
            dry_run: True の場合は実際の送信を行わない

        Returns:
            送信結果を表す NotificationResult
        """
        ...

    def is_available(self) -> bool:
        """プロバイダーが利用可能な状態か確認する（ヘルスチェック）。"""
        # デフォルト実装は常に True を返す（サブクラスでオーバーライド可能）
        return True


# ---------------------------------------------------------------------------
# MattermostProvider クラス
# ---------------------------------------------------------------------------

class MattermostProvider(NotificationProvider):
    """Mattermost Incoming Webhook を使った通知プロバイダー。"""

    def __init__(
        self,
        webhook_url: str,
        channel: str = "",
        username: str = "k1s0-escalation",
        timeout_s: int = 10,
    ) -> None:
        """MattermostProvider を初期化する。"""
        # Webhook URL を保持する
        self._webhook_url = webhook_url
        # 投稿先チャンネル名を保持する（空の場合はデフォルトチャンネルを使用する）
        self._channel = channel
        # Bot ユーザー名を保持する
        self._username = username
        # HTTP リクエストのタイムアウト秒数を保持する
        self._timeout_s = timeout_s

    @property
    def name(self) -> str:
        """プロバイダー名を返す。"""
        return "mattermost"

    def _build_message(self, payload: NotificationPayload) -> dict[str, Any]:
        """Mattermost メッセージのペイロードを構築する。"""
        # 重要度に応じて絵文字プレフィクスを選択する
        severity_emoji = {
            "critical": ":red_circle:",
            "warning": ":warning:",
            "info": ":information_source:",
        }.get(payload.severity, ":white_circle:")
        # signal_class に応じてカラーを選択する
        color_map = {
            "v1_page": "#FF0000",
            "v1_freeze": "#FF6600",
            "v1_slo_breach": "#FF6600",
            "v1_notify": "#FFC000",
            "v1_audit": "#0080FF",
        }
        color = color_map.get(payload.signal_class, "#808080")
        # attachment 形式でメッセージを構築する
        attachment = {
            "color": color,
            "title": f"{severity_emoji} {payload.title}",
            "text": payload.body,
            "fields": [
                {"short": True, "title": "Signal Class", "value": payload.signal_class},
                {"short": True, "title": "Severity", "value": payload.severity},
                {"short": True, "title": "Incident ID", "value": payload.incident_id},
                {"short": True, "title": "HLC", "value": payload.hlc_ts},
            ],
        }
        # runbook URL が設定されている場合はフィールドに追加する
        if payload.runbook_url:
            attachment["fields"].append({"short": False, "title": "Runbook", "value": payload.runbook_url})
        # メッセージ辞書を構築して返す
        msg: dict[str, Any] = {
            "username": self._username,
            "attachments": [attachment],
        }
        # チャンネルが指定されている場合は追加する
        if self._channel:
            msg["channel"] = self._channel
        return msg

    def send(self, payload: NotificationPayload, dry_run: bool = False) -> NotificationResult:
        """Mattermost Incoming Webhook に通知を送信する。"""
        # dry-run モードの場合は実際の送信を行わない
        if dry_run:
            logger.info("MattermostProvider: dry-run 送信 alert=%s", payload.alert_name)
            return NotificationResult(
                success=True, provider_name=self.name, status_code=0, dry_run=True
            )
        # webhook_url が未設定の場合はエラーを返す
        if not self._webhook_url:
            return NotificationResult(
                success=False, provider_name=self.name,
                error_message="webhook_url が未設定"
            )
        # メッセージペイロードを構築する
        msg = self._build_message(payload)
        # JSON エンコードする
        body = json.dumps(msg).encode("utf-8")
        # HTTP POST リクエストを送信する
        try:
            req = urllib.request.Request(
                self._webhook_url,
                data=body,
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            with urllib.request.urlopen(req, timeout=self._timeout_s) as resp:
                status = resp.status
            # ステータス 200 の場合は成功として返す
            if status == 200:
                logger.info("MattermostProvider: 送信成功 alert=%s status=%d", payload.alert_name, status)
                return NotificationResult(success=True, provider_name=self.name, status_code=status)
            # 200 以外はエラーとして扱う
            return NotificationResult(
                success=False, provider_name=self.name, status_code=status,
                error_message=f"HTTP {status}"
            )
        except urllib.error.HTTPError as exc:
            logger.error("MattermostProvider: HTTP エラー %d", exc.code)
            return NotificationResult(
                success=False, provider_name=self.name, status_code=exc.code,
                error_message=str(exc)
            )
        except Exception as exc:
            logger.error("MattermostProvider: 送信エラー %s", exc)
            return NotificationResult(
                success=False, provider_name=self.name, error_message=str(exc)
            )


# ---------------------------------------------------------------------------
# SlackProvider クラス
# ---------------------------------------------------------------------------

class SlackProvider(NotificationProvider):
    """Slack Incoming Webhook を使った通知プロバイダー。"""

    def __init__(
        self,
        webhook_url: str,
        channel: str = "",
        username: str = "k1s0-escalation",
        timeout_s: int = 10,
    ) -> None:
        """SlackProvider を初期化する。"""
        # Webhook URL を保持する
        self._webhook_url = webhook_url
        # 投稿先チャンネル名を保持する
        self._channel = channel
        # Bot ユーザー名を保持する
        self._username = username
        # HTTP リクエストのタイムアウト秒数を保持する
        self._timeout_s = timeout_s

    @property
    def name(self) -> str:
        """プロバイダー名を返す。"""
        return "slack"

    def _build_message(self, payload: NotificationPayload) -> dict[str, Any]:
        """Slack Block Kit 形式のメッセージを構築する。"""
        # 重要度に応じてカラーコードを選択する
        color_map = {
            "critical": "#FF0000",
            "warning": "#FFC000",
            "info": "#36A64F",
        }
        color = color_map.get(payload.severity, "#808080")
        # Slack attachment 形式のメッセージを構築する
        attachment = {
            "color": color,
            "title": payload.title,
            "text": payload.body,
            "fields": [
                {"short": True, "title": "Signal Class", "value": payload.signal_class},
                {"short": True, "title": "Severity", "value": payload.severity},
                {"short": True, "title": "Incident", "value": payload.incident_id},
                {"short": True, "title": "HLC TS", "value": payload.hlc_ts},
            ],
            "footer": "k1s0 escalation engine",
            "ts": int(time.time()),
        }
        # runbook URL が設定されている場合はフィールドに追加する
        if payload.runbook_url:
            attachment["title_link"] = payload.runbook_url
        # メッセージ辞書を構築する
        msg: dict[str, Any] = {
            "username": self._username,
            "attachments": [attachment],
        }
        # チャンネルが指定されている場合は追加する
        if self._channel:
            msg["channel"] = self._channel
        return msg

    def send(self, payload: NotificationPayload, dry_run: bool = False) -> NotificationResult:
        """Slack Incoming Webhook に通知を送信する。"""
        # dry-run モードの場合は実際の送信を行わない
        if dry_run:
            logger.info("SlackProvider: dry-run 送信 alert=%s", payload.alert_name)
            return NotificationResult(
                success=True, provider_name=self.name, status_code=0, dry_run=True
            )
        # webhook_url が未設定の場合はエラーを返す
        if not self._webhook_url:
            return NotificationResult(
                success=False, provider_name=self.name,
                error_message="webhook_url が未設定"
            )
        # メッセージを構築してエンコードする
        msg = self._build_message(payload)
        body = json.dumps(msg).encode("utf-8")
        # HTTP POST リクエストを Slack Webhook に送信する
        try:
            req = urllib.request.Request(
                self._webhook_url,
                data=body,
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            with urllib.request.urlopen(req, timeout=self._timeout_s) as resp:
                status = resp.status
                # Slack は成功時に "ok" を返す
                resp_body = resp.read().decode("utf-8", errors="replace")
            # ステータス 200 かつレスポンスが "ok" の場合は成功
            if status == 200 and resp_body.strip() == "ok":
                logger.info("SlackProvider: 送信成功 alert=%s", payload.alert_name)
                return NotificationResult(success=True, provider_name=self.name, status_code=status)
            return NotificationResult(
                success=False, provider_name=self.name, status_code=status,
                error_message=f"Slack response: {resp_body[:100]}"
            )
        except Exception as exc:
            logger.error("SlackProvider: 送信エラー %s", exc)
            return NotificationResult(
                success=False, provider_name=self.name, error_message=str(exc)
            )


# ---------------------------------------------------------------------------
# PagerDutyProvider クラス
# ---------------------------------------------------------------------------

class PagerDutyProvider(NotificationProvider):
    """PagerDuty Events API v2 を使った通知プロバイダー。"""

    # PagerDuty Events API v2 のエンドポイント
    _EVENTS_API_URL = "https://events.pagerduty.com/v2/enqueue"

    def __init__(
        self,
        integration_key: str,
        timeout_s: int = 15,
    ) -> None:
        """PagerDutyProvider を初期化する。"""
        # PagerDuty integration key を保持する
        self._integration_key = integration_key
        # HTTP リクエストのタイムアウト秒数を保持する
        self._timeout_s = timeout_s

    @property
    def name(self) -> str:
        """プロバイダー名を返す。"""
        return "pagerduty"

    def _map_severity(self, severity: str) -> str:
        """k1s0 重要度を PagerDuty severity にマッピングする。"""
        # PagerDuty が受け入れる severity は critical / error / warning / info
        mapping = {
            "critical": "critical",
            "warning": "warning",
            "info": "info",
        }
        return mapping.get(severity, "error")

    def _build_event(self, payload: NotificationPayload, action: str = "trigger") -> dict[str, Any]:
        """PagerDuty Events API v2 形式のイベントペイロードを構築する。"""
        # PD イベントの payload セクションを構築する
        pd_payload: dict[str, Any] = {
            "summary": payload.title,
            "severity": self._map_severity(payload.severity),
            "source": "k1s0-escalation-engine",
            "component": payload.signal_class,
            "group": payload.incident_id,
            "class": payload.signal_class,
            "custom_details": {
                "body": payload.body,
                "hlc_ts": payload.hlc_ts,
                "alert_name": payload.alert_name,
                "runbook_url": payload.runbook_url,
            },
        }
        # イベント辞書を構築して返す
        event: dict[str, Any] = {
            "routing_key": self._integration_key,
            "event_action": action,
            "dedup_key": payload.alert_fingerprint,
            "payload": pd_payload,
        }
        # runbook URL がある場合はリンクとして追加する
        if payload.runbook_url:
            event["links"] = [{"href": payload.runbook_url, "text": "Runbook"}]
        return event

    def send(self, payload: NotificationPayload, dry_run: bool = False) -> NotificationResult:
        """PagerDuty Events API v2 に通知を送信する。"""
        # dry-run モードの場合は実際の送信を行わない
        if dry_run:
            logger.info("PagerDutyProvider: dry-run 送信 alert=%s", payload.alert_name)
            return NotificationResult(
                success=True, provider_name=self.name, status_code=0, dry_run=True
            )
        # integration_key が未設定の場合はエラーを返す
        if not self._integration_key:
            return NotificationResult(
                success=False, provider_name=self.name,
                error_message="integration_key が未設定"
            )
        # イベントペイロードを構築してエンコードする
        event = self._build_event(payload)
        body = json.dumps(event).encode("utf-8")
        # HTTP POST リクエストを PagerDuty API に送信する
        try:
            req = urllib.request.Request(
                self._EVENTS_API_URL,
                data=body,
                headers={
                    "Content-Type": "application/json",
                    "X-Routing-Key": self._integration_key,
                },
                method="POST",
            )
            with urllib.request.urlopen(req, timeout=self._timeout_s) as resp:
                status = resp.status
                resp_body = resp.read().decode("utf-8", errors="replace")
            # ステータス 202 が PagerDuty の成功コード
            if status == 202:
                logger.info("PagerDutyProvider: 送信成功 alert=%s", payload.alert_name)
                return NotificationResult(success=True, provider_name=self.name, status_code=status)
            return NotificationResult(
                success=False, provider_name=self.name, status_code=status,
                error_message=f"PD response {status}: {resp_body[:200]}"
            )
        except urllib.error.HTTPError as exc:
            logger.error("PagerDutyProvider: HTTP エラー %d", exc.code)
            return NotificationResult(
                success=False, provider_name=self.name, status_code=exc.code,
                error_message=str(exc)
            )
        except Exception as exc:
            logger.error("PagerDutyProvider: 送信エラー %s", exc)
            return NotificationResult(
                success=False, provider_name=self.name, error_message=str(exc)
            )

    def resolve(self, alert_fingerprint: str, dry_run: bool = False) -> NotificationResult:
        """PagerDuty でインシデントを解決済みにする。

        Args:
            alert_fingerprint: 解決するアラートのフィンガープリント（dedup_key）
            dry_run: True の場合は実際の送信を行わない

        Returns:
            送信結果を表す NotificationResult
        """
        # dry-run モードの場合は実際の送信を行わない
        if dry_run:
            logger.info("PagerDutyProvider: dry-run resolve fingerprint=%s", alert_fingerprint)
            return NotificationResult(success=True, provider_name=self.name, dry_run=True)
        # 解決イベントのペイロードを構築する
        event = {
            "routing_key": self._integration_key,
            "event_action": "resolve",
            "dedup_key": alert_fingerprint,
        }
        body = json.dumps(event).encode("utf-8")
        # HTTP POST で解決イベントを送信する
        try:
            req = urllib.request.Request(
                self._EVENTS_API_URL,
                data=body,
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            with urllib.request.urlopen(req, timeout=self._timeout_s) as resp:
                status = resp.status
            return NotificationResult(
                success=(status == 202), provider_name=self.name, status_code=status
            )
        except Exception as exc:
            logger.error("PagerDutyProvider: resolve エラー %s", exc)
            return NotificationResult(success=False, provider_name=self.name, error_message=str(exc))


# ---------------------------------------------------------------------------
# RoutingRule データクラス
# ---------------------------------------------------------------------------

@dataclass
class RoutingRule:
    """signal_class と severity に基づくルーティングルールを定義する。"""

    # マッチする signal_class（"*" でワイルドカード）
    signal_class: str
    # マッチする severity（"*" でワイルドカード）
    severity: str
    # プライマリプロバイダーのリスト（順番通りに試行する）
    primary_providers: list[str]
    # フォールバックプロバイダーのリスト
    fallback_providers: list[str] = field(default_factory=list)
    # ルール有効フラグ
    enabled: bool = True

    def matches(self, signal_class: str, severity: str) -> bool:
        """このルールが指定の signal_class と severity にマッチするか確認する。"""
        # signal_class のマッチ確認（"*" はワイルドカード）
        sc_match = self.signal_class == "*" or self.signal_class == signal_class
        # severity のマッチ確認（"*" はワイルドカード）
        sev_match = self.severity == "*" or self.severity == severity
        # 両方マッチかつ有効な場合に True を返す
        return sc_match and sev_match and self.enabled


# ---------------------------------------------------------------------------
# DedupCache クラス
# ---------------------------------------------------------------------------

class DedupCache:
    """アラートフィンガープリントの重複排除キャッシュ。"""

    def __init__(self, window_s: int = _DEDUP_WINDOW_S) -> None:
        """DedupCache を初期化する。"""
        # 重複排除ウィンドウ秒数を保持する
        self._window_s = window_s
        # fingerprint → 最後の送信時刻のマッピングを初期化する
        self._cache: dict[str, float] = {}

    def is_duplicate(self, fingerprint: str, now: float | None = None) -> bool:
        """指定フィンガープリントが重複かどうか確認する。

        Args:
            fingerprint: 確認するアラートフィンガープリント
            now: 現在時刻（テスト用にオーバーライド可能）

        Returns:
            重複（5 分以内に同一フィンガープリントを送信済み）の場合 True
        """
        # 現在時刻を取得する
        current = now if now is not None else time.time()
        # キャッシュにエントリが存在しない場合は重複ではない
        if fingerprint not in self._cache:
            return False
        # ウィンドウ内に送信済みの場合は重複と判定する
        last_sent = self._cache[fingerprint]
        return (current - last_sent) < self._window_s

    def mark_sent(self, fingerprint: str, now: float | None = None) -> None:
        """フィンガープリントを送信済みとしてキャッシュに記録する。"""
        # 現在時刻を記録する
        current = now if now is not None else time.time()
        self._cache[fingerprint] = current

    def evict_expired(self, now: float | None = None) -> int:
        """期限切れのキャッシュエントリを削除してメモリを解放する。

        Returns:
            削除したエントリの件数
        """
        # 現在時刻を取得する
        current = now if now is not None else time.time()
        # 期限切れエントリのキーを収集する
        expired_keys = [
            fp for fp, ts in self._cache.items()
            if (current - ts) >= self._window_s
        ]
        # 期限切れエントリを削除する
        for fp in expired_keys:
            del self._cache[fp]
        return len(expired_keys)

    def size(self) -> int:
        """キャッシュの現在エントリ数を返す。"""
        return len(self._cache)


# ---------------------------------------------------------------------------
# ProviderRouter クラス
# ---------------------------------------------------------------------------

class ProviderRouter:
    """signal_class と severity に基づいて通知プロバイダーにルーティングするルーター。

    ルーティングルールは YAML 設定ファイルまたはプログラム的に登録する。
    プライマリプロバイダーが失敗した場合はフォールバックプロバイダーを試行する。
    同一 alert_fingerprint は dedup_window_s 以内に再通知しない。
    """

    def __init__(self, dedup_window_s: int = _DEDUP_WINDOW_S, dry_run: bool = False) -> None:
        """ProviderRouter を初期化する。"""
        # プロバイダー名 → NotificationProvider のマッピングを初期化する
        self._providers: dict[str, NotificationProvider] = {}
        # ルーティングルールリストを初期化する（登録順に評価する）
        self._rules: list[RoutingRule] = []
        # 重複排除キャッシュを初期化する
        self._dedup = DedupCache(window_s=dedup_window_s)
        # dry-run モードフラグを保持する
        self._dry_run = dry_run
        # 送信成功件数カウンタを初期化する
        self._routed_count: int = 0
        # 重複排除でスキップした件数カウンタを初期化する
        self._dedup_skipped_count: int = 0
        # フォールバック使用件数カウンタを初期化する
        self._fallback_count: int = 0

    def register_provider(self, provider: NotificationProvider) -> None:
        """通知プロバイダーをルーターに登録する。

        Args:
            provider: 登録する NotificationProvider インスタンス
        """
        # プロバイダー名をキーとして登録する
        self._providers[provider.name] = provider
        logger.debug("ProviderRouter: プロバイダー登録 name=%s", provider.name)

    def add_rule(self, rule: RoutingRule) -> None:
        """ルーティングルールをルーターに追加する（後から追加したルールが低優先度）。

        Args:
            rule: 追加する RoutingRule
        """
        # ルールリストの末尾に追加する
        self._rules.append(rule)
        logger.debug(
            "ProviderRouter: ルール追加 signal_class=%s severity=%s providers=%s",
            rule.signal_class, rule.severity, rule.primary_providers
        )

    def add_default_rules(self) -> None:
        """製造業 k1s0 プラットフォーム用のデフォルトルーティングルールを追加する。"""
        # v1_page（最高優先度ページング）: Mattermost + PagerDuty を使用する
        self.add_rule(RoutingRule(
            signal_class="v1_page",
            severity="critical",
            primary_providers=["mattermost", "pagerduty"],
            fallback_providers=["slack"],
        ))
        # v1_freeze（システムフリーズ）: Mattermost + PagerDuty を使用する
        self.add_rule(RoutingRule(
            signal_class="v1_freeze",
            severity="critical",
            primary_providers=["pagerduty", "mattermost"],
            fallback_providers=["slack"],
        ))
        # v1_slo_breach（SLO 違反）: Mattermost を主に使用する
        self.add_rule(RoutingRule(
            signal_class="v1_slo_breach",
            severity="warning",
            primary_providers=["mattermost"],
            fallback_providers=["slack"],
        ))
        # v1_notify（一般通知）: Slack を使用する
        self.add_rule(RoutingRule(
            signal_class="v1_notify",
            severity="info",
            primary_providers=["slack"],
            fallback_providers=["mattermost"],
        ))
        # v1_audit（監査ログ）: Mattermost のみを使用する
        self.add_rule(RoutingRule(
            signal_class="v1_audit",
            severity="info",
            primary_providers=["mattermost"],
            fallback_providers=[],
        ))
        # ワイルドカード（その他の全ての通知）: Mattermost にフォールバックする
        self.add_rule(RoutingRule(
            signal_class="*",
            severity="*",
            primary_providers=["mattermost"],
            fallback_providers=[],
        ))

    def route(self, payload: NotificationPayload) -> list[NotificationResult]:
        """通知ペイロードをルーティングして適切なプロバイダーに送信する。

        1. alert_fingerprint で重複チェックを実行する
        2. マッチするルールを検索する
        3. プライマリプロバイダーに順番に送信を試みる
        4. 全プライマリが失敗した場合はフォールバックを試みる

        Args:
            payload: 送信する通知ペイロード

        Returns:
            送信試行の結果リスト（複数プロバイダーへの送信を含む）
        """
        # 重複チェックを実行する
        if self._dedup.is_duplicate(payload.alert_fingerprint):
            logger.info(
                "ProviderRouter: 重複排除スキップ fingerprint=%s", payload.alert_fingerprint
            )
            self._dedup_skipped_count += 1
            return []
        # マッチするルールを検索する
        matched_rule = self._find_rule(payload.signal_class, payload.severity)
        # ルールが見つからない場合は空リストを返す
        if matched_rule is None:
            logger.warning(
                "ProviderRouter: ルールなし signal_class=%s severity=%s",
                payload.signal_class, payload.severity
            )
            return []
        # 結果リストを初期化する
        results: list[NotificationResult] = []
        # プライマリプロバイダーへの送信を試みる
        primary_success = False
        for provider_name in matched_rule.primary_providers:
            result = self._send_to_provider(provider_name, payload)
            results.append(result)
            # 1 件でも成功したら終了する
            if result.success:
                primary_success = True
                break
        # プライマリが全て失敗した場合はフォールバックを試みる
        if not primary_success and matched_rule.fallback_providers:
            self._fallback_count += 1
            logger.warning(
                "ProviderRouter: フォールバック試行 signal_class=%s", payload.signal_class
            )
            for provider_name in matched_rule.fallback_providers:
                result = self._send_to_provider(provider_name, payload)
                results.append(result)
                # フォールバックで 1 件でも成功したら終了する
                if result.success:
                    break
        # 送信済みとしてキャッシュに記録する
        self._dedup.mark_sent(payload.alert_fingerprint)
        # ルーティング件数をインクリメントする
        self._routed_count += 1
        return results

    def _find_rule(self, signal_class: str, severity: str) -> RoutingRule | None:
        """最初にマッチするルーティングルールを返す。"""
        # ルールリストを順番に評価する
        for rule in self._rules:
            if rule.matches(signal_class, severity):
                return rule
        return None

    def _send_to_provider(self, provider_name: str, payload: NotificationPayload) -> NotificationResult:
        """指定プロバイダーに通知を送信する（エラーは NotificationResult に包む）。"""
        # プロバイダーが登録されているか確認する
        if provider_name not in self._providers:
            logger.error("ProviderRouter: プロバイダー未登録 name=%s", provider_name)
            return NotificationResult(
                success=False, provider_name=provider_name,
                error_message=f"プロバイダー未登録: {provider_name}"
            )
        # プロバイダーを取得して送信を試みる
        provider = self._providers[provider_name]
        try:
            return provider.send(payload, dry_run=self._dry_run)
        except Exception as exc:
            logger.error("ProviderRouter: プロバイダー例外 name=%s error=%s", provider_name, exc)
            return NotificationResult(
                success=False, provider_name=provider_name, error_message=str(exc)
            )

    def get_metrics(self) -> dict[str, Any]:
        """ルーターのメトリクスを辞書形式で返す。"""
        return {
            "routed_count": self._routed_count,
            "dedup_skipped_count": self._dedup_skipped_count,
            "fallback_count": self._fallback_count,
            "dedup_cache_size": self._dedup.size(),
            "registered_providers": list(self._providers.keys()),
            "rule_count": len(self._rules),
        }

    @staticmethod
    def compute_fingerprint(alert_name: str, labels: dict[str, str]) -> str:
        """アラート名とラベルからフィンガープリントを計算する。

        Args:
            alert_name: アラート名
            labels: アラートのラベル辞書

        Returns:
            SHA-256 の先頭 16 文字のフィンガープリント文字列
        """
        # ラベルをソートして一貫した文字列を生成する
        sorted_labels = sorted(labels.items())
        raw = f"{alert_name}:{sorted_labels}"
        # SHA-256 ハッシュの先頭 16 文字を返す
        return hashlib.sha256(raw.encode("utf-8")).hexdigest()[:16]
