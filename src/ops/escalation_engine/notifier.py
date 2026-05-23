"""src/ops/escalation_engine/notifier.py

Mattermost / 汎用 webhook への通知クライアント。
仕様: 17_運用ループ適合仕様.md §self_escalation_engine

- phase_1_detect: on-call へのページング
- phase_2_triage: incident チャンネル作成
- 3 階層 escalation tier の通知
- dry_run モードで実際の HTTP リクエストを抑制可能
"""

from __future__ import annotations

import json
import logging
import os
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any

logger = logging.getLogger(__name__)

_DRY_RUN_DEFAULT = os.environ.get("ESCALATION_DRY_RUN", "false").lower() in ("1", "true", "yes")


@dataclass
class NotificationResult:
    success: bool
    status_code: int
    message: str
    dry_run: bool = False


@dataclass
class EscalationTier:
    tier_num: int           # 1 / 2 / 3
    label: str              # "on-call" / "senior on-call" / "incident commander"
    channel: str            # Mattermost チャンネル名
    mention: str            # @mention 対象（@oncall / @sre-lead 等）
    ack_timeout_seconds: int


_DEFAULT_TIERS: list[EscalationTier] = [
    EscalationTier(1, "on-call engineer", "ops-incidents", "@oncall", 300),
    EscalationTier(2, "senior on-call", "ops-escalation", "@sre-lead", 600),
    EscalationTier(3, "incident commander", "ops-p0", "@incident-commander", 1200),
]


class MattermostNotifier:
    """Mattermost Incoming Webhook 経由で通知するクライアント。

    環境変数:
      MATTERMOST_WEBHOOK_URL: Incoming Webhook URL（未設定時は dry-run）
      ESCALATION_DRY_RUN: "true" の場合 HTTP 送信をスキップ
    """

    def __init__(
        self,
        webhook_url: str | None = None,
        dry_run: bool | None = None,
        tiers: list[EscalationTier] | None = None,
    ) -> None:
        self._webhook_url = webhook_url or os.environ.get("MATTERMOST_WEBHOOK_URL", "")
        self._dry_run = dry_run if dry_run is not None else _DRY_RUN_DEFAULT
        if not self._webhook_url:
            logger.warning("MATTERMOST_WEBHOOK_URL not set; escalation_engine will dry-run")
            self._dry_run = True
        self._tiers = tiers or _DEFAULT_TIERS

    def _post(self, payload: dict[str, Any]) -> NotificationResult:
        """Mattermost Incoming Webhook に HTTP POST する。"""
        if self._dry_run:
            logger.info("[DRY-RUN] Mattermost payload: %s", json.dumps(payload, ensure_ascii=False))
            return NotificationResult(True, 200, "dry-run", dry_run=True)

        body = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            self._webhook_url,
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                status = resp.status
                msg = resp.read().decode("utf-8", errors="replace")
                if status == 200:
                    return NotificationResult(True, status, msg)
                return NotificationResult(False, status, f"Unexpected status: {status}")
        except urllib.error.URLError as exc:
            logger.error("Mattermost POST failed: %s", exc)
            return NotificationResult(False, 0, str(exc))

    def page_oncall(
        self,
        incident_id: str,
        signal_class: str,
        alertname: str,
        runbook_ref: str,
        severity: str,
        tier: int = 1,
    ) -> NotificationResult:
        """on-call にページングする。"""
        t = self._tier(tier)
        ts = datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        payload = {
            "channel": t.channel,
            "username": "escalation-engine",
            "text": (
                f"### :red_circle: [{severity.upper()}] {alertname}\n"
                f"**Incident:** `{incident_id}`  |  **Signal:** `{signal_class}`\n"
                f"**Tier {t.tier_num} Page** → {t.mention}\n"
                f"**Runbook:** {runbook_ref}\n"
                f"**Time:** {ts}\n"
                f"Ack within **{t.ack_timeout_seconds // 60} min** or escalation to Tier {tier + 1}."
            ),
        }
        result = self._post(payload)
        logger.info(
            "page_oncall incident=%s tier=%d severity=%s dry_run=%s status=%d",
            incident_id, tier, severity, result.dry_run, result.status_code,
        )
        return result

    def notify_triage_open(
        self,
        incident_id: str,
        signal_class: str,
        alertname: str,
        tier: int = 1,
    ) -> NotificationResult:
        """phase_2_triage: インシデントチャンネルを開いた旨を通知する。"""
        t = self._tier(tier)
        payload = {
            "channel": t.channel,
            "username": "escalation-engine",
            "text": (
                f":mag: **Triage open** — `{incident_id}`\n"
                f"Signal: `{signal_class}` | Alert: `{alertname}`\n"
                "Confirm incident class and create IR ticket within **30 min**."
            ),
        }
        return self._post(payload)

    def notify_resolved(self, incident_id: str, resolution_note: str = "") -> NotificationResult:
        """インシデント解決通知。"""
        payload = {
            "channel": "ops-incidents",
            "username": "escalation-engine",
            "text": (
                f":white_check_mark: **Resolved** — `{incident_id}`\n"
                + (f"Note: {resolution_note}" if resolution_note else "")
            ),
        }
        return self._post(payload)

    def notify_escalation(
        self,
        incident_id: str,
        from_tier: int,
        to_tier: int,
        reason: str = "ack timeout",
    ) -> NotificationResult:
        """escalation DAG の tier 昇格通知。"""
        to = self._tier(to_tier)
        payload = {
            "channel": to.channel,
            "username": "escalation-engine",
            "text": (
                f":rotating_light: **Escalation** Tier {from_tier} → Tier {to_tier}\n"
                f"Incident: `{incident_id}` | Reason: {reason}\n"
                f"{to.mention} please acknowledge immediately."
            ),
        }
        result = self._post(payload)
        logger.warning(
            "escalation incident=%s %d->%d reason=%s dry_run=%s",
            incident_id, from_tier, to_tier, reason, result.dry_run,
        )
        return result

    def _tier(self, tier_num: int) -> EscalationTier:
        for t in self._tiers:
            if t.tier_num == tier_num:
                return t
        return self._tiers[-1]
