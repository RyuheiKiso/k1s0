"""src/ops/escalation_engine/engine.py

self_escalation_engine のメインエントリポイント。

仕様: 17_運用ループ適合仕様.md §self_escalation_engine
- Alertmanager webhook を受信 (POST /webhook)
- signal_class を判定して runbook を引当
- 3 階層 escalation DAG で Mattermost page → ack 監視 → 上位 tier エスカレーション
- 5 signal_class 全てを担当
- ack_window: v1_page 5 分 / v1_freeze 3 分 / v1_notify 60 分

起動方法:
  python -m src.ops.escalation_engine.engine [--port 9093] [--dry-run]
  または
  python src/ops/escalation_engine/engine.py --port 9093

環境変数:
  MATTERMOST_WEBHOOK_URL: Incoming Webhook URL
  ESCALATION_DRY_RUN: "true" で dry-run モード（通知を送らない）
  ESCALATION_PORT: サーバーポート（デフォルト 9093）
"""

from __future__ import annotations

import argparse
import hashlib
import http.server
import json
import logging
import os
import sys
import threading
import time
import uuid
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .notifier import MattermostNotifier, NotificationResult
from .signal_classifier import SignalClass, SignalClassifier

logger = logging.getLogger(__name__)

_REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent


# ---------------------------------------------------------------------------
# Incident state machine
# ---------------------------------------------------------------------------

class IncidentState:
    FIRING = "firing"
    ACKNOWLEDGED = "acknowledged"
    MITIGATED = "mitigated"
    RESOLVED = "resolved"


@dataclass
class Incident:
    incident_id: str
    signal_class: str
    alertname: str
    alert_labels: dict[str, Any]
    severity: str
    runbook_ref: str
    state: str = IncidentState.FIRING
    current_tier: int = 1
    fired_at: float = field(default_factory=time.time)
    acked_at: float | None = None
    resolved_at: float | None = None
    escalation_history: list[dict[str, Any]] = field(default_factory=list)

    def age_seconds(self) -> float:
        return time.time() - self.fired_at

    def is_ack_overdue(self, ack_window_seconds: int) -> bool:
        if self.state != IncidentState.FIRING:
            return False
        return self.age_seconds() > ack_window_seconds

    def acknowledge(self, by: str = "operator") -> None:
        self.state = IncidentState.ACKNOWLEDGED
        self.acked_at = time.time()
        logger.info("incident %s acked by %s after %.0fs", self.incident_id, by, self.age_seconds())

    def resolve(self, note: str = "") -> None:
        self.state = IncidentState.RESOLVED
        self.resolved_at = time.time()
        logger.info("incident %s resolved: %s", self.incident_id, note)


# ---------------------------------------------------------------------------
# Runbook catalog lookup
# ---------------------------------------------------------------------------

def _lookup_runbook(signal_class: str) -> str:
    """signal_class から runbook 参照 URL / 名前を引当する。

    runbook_catalog/ 配下の Markdown ファイル名を返す。
    """
    runbook_dir = Path(__file__).parent.parent / "runbook_catalog"
    candidate = runbook_dir / f"{signal_class}.md"
    if candidate.exists():
        return f"ops/runbook_catalog/{signal_class}.md"
    return f"ops/runbook_catalog/{signal_class}.md (pending)"


# ---------------------------------------------------------------------------
# Incident registry
# ---------------------------------------------------------------------------

class IncidentRegistry:
    """発生中インシデントのスレッドセーフな管理。"""

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._incidents: dict[str, Incident] = {}

    def register(self, incident: Incident) -> None:
        with self._lock:
            self._incidents[incident.incident_id] = incident

    def get(self, incident_id: str) -> Incident | None:
        with self._lock:
            return self._incidents.get(incident_id)

    def acknowledge(self, incident_id: str, by: str = "operator") -> bool:
        with self._lock:
            inc = self._incidents.get(incident_id)
            if inc and inc.state == IncidentState.FIRING:
                inc.acknowledge(by)
                return True
            return False

    def resolve(self, incident_id: str, note: str = "") -> bool:
        with self._lock:
            inc = self._incidents.get(incident_id)
            if inc and inc.state not in (IncidentState.RESOLVED,):
                inc.resolve(note)
                return True
            return False

    def firing_incidents(self) -> list[Incident]:
        with self._lock:
            return [i for i in self._incidents.values() if i.state == IncidentState.FIRING]

    def all_incidents(self) -> list[dict[str, Any]]:
        with self._lock:
            return [
                {
                    "incident_id": i.incident_id,
                    "signal_class": i.signal_class,
                    "alertname": i.alertname,
                    "state": i.state,
                    "severity": i.severity,
                    "current_tier": i.current_tier,
                    "age_seconds": round(i.age_seconds(), 1),
                }
                for i in self._incidents.values()
            ]


# ---------------------------------------------------------------------------
# Escalation DAG monitor
# ---------------------------------------------------------------------------

_MAX_TIER = 3


class EscalationMonitor:
    """バックグラウンドスレッドで ack_window を監視し、timeout 時に tier を昇格する。"""

    def __init__(
        self,
        registry: IncidentRegistry,
        notifier: MattermostNotifier,
        classifier: SignalClassifier,
        poll_interval: float = 10.0,
    ) -> None:
        self._registry = registry
        self._notifier = notifier
        self._classifier = classifier
        self._poll_interval = poll_interval
        self._running = False
        self._thread: threading.Thread | None = None

    def start(self) -> None:
        self._running = True
        self._thread = threading.Thread(target=self._loop, daemon=True, name="escalation-monitor")
        self._thread.start()
        logger.info("EscalationMonitor started (poll_interval=%.1fs)", self._poll_interval)

    def stop(self) -> None:
        self._running = False
        if self._thread:
            self._thread.join(timeout=5.0)

    def _loop(self) -> None:
        while self._running:
            self._check_all()
            time.sleep(self._poll_interval)

    def _check_all(self) -> None:
        for incident in self._registry.firing_incidents():
            sc = self._classifier.get(incident.signal_class)
            if sc is None:
                continue
            if incident.is_ack_overdue(sc.ack_window_seconds):
                self._escalate(incident, sc)

    def _escalate(self, incident: Incident, sc: SignalClass) -> None:
        next_tier = incident.current_tier + 1
        if next_tier > _MAX_TIER:
            logger.error(
                "incident %s exceeded max tier %d — no further escalation",
                incident.incident_id, _MAX_TIER,
            )
            return

        from_tier = incident.current_tier
        incident.current_tier = next_tier
        incident.escalation_history.append({
            "from_tier": from_tier,
            "to_tier": next_tier,
            "at": datetime.now(tz=timezone.utc).isoformat(),
            "reason": "ack_timeout",
        })

        self._notifier.notify_escalation(
            incident_id=incident.incident_id,
            from_tier=from_tier,
            to_tier=next_tier,
        )
        self._notifier.page_oncall(
            incident_id=incident.incident_id,
            signal_class=incident.signal_class,
            alertname=incident.alertname,
            runbook_ref=incident.runbook_ref,
            severity=incident.severity,
            tier=next_tier,
        )
        logger.warning(
            "escalated incident=%s %d->%d", incident.incident_id, from_tier, next_tier
        )


# ---------------------------------------------------------------------------
# HTTP webhook handler
# ---------------------------------------------------------------------------

def _make_incident_id(alertname: str, labels: dict[str, Any]) -> str:
    """alert の fingerprint から重複排除用 incident_id を生成する。"""
    key = alertname + json.dumps(labels, sort_keys=True)
    return "INC-" + hashlib.sha256(key.encode()).hexdigest()[:12].upper()


class WebhookHandler(http.server.BaseHTTPRequestHandler):
    """Alertmanager webhook + ack / resolve / status API を処理する。

    POST /webhook        — Alertmanager fires alert
    POST /ack/{id}       — acknowledge incident
    POST /resolve/{id}   — resolve incident
    GET  /status         — all incidents JSON
    """

    registry: IncidentRegistry
    notifier: MattermostNotifier
    classifier: SignalClassifier

    def log_message(self, format: str, *args: Any) -> None:
        logger.debug(format, *args)

    def do_GET(self) -> None:
        if self.path == "/status":
            self._respond_json(200, {"incidents": self.registry.all_incidents()})
        elif self.path == "/healthz":
            self._respond_json(200, {"status": "ok"})
        else:
            self._respond_json(404, {"error": "not found"})

    def do_POST(self) -> None:
        if self.path == "/webhook":
            self._handle_webhook()
        elif self.path.startswith("/ack/"):
            incident_id = self.path[len("/ack/"):]
            ok = self.registry.acknowledge(incident_id)
            self._respond_json(200 if ok else 404, {"acked": ok, "incident_id": incident_id})
        elif self.path.startswith("/resolve/"):
            incident_id = self.path[len("/resolve/"):]
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length).decode("utf-8", errors="replace") if content_length else "{}"
            note = json.loads(body).get("note", "") if body else ""
            ok = self.registry.resolve(incident_id, note)
            if ok:
                self.notifier.notify_resolved(incident_id, note)
            self._respond_json(200 if ok else 404, {"resolved": ok, "incident_id": incident_id})
        else:
            self._respond_json(404, {"error": "not found"})

    def _handle_webhook(self) -> None:
        content_length = int(self.headers.get("Content-Length", 0))
        if content_length == 0:
            self._respond_json(400, {"error": "empty body"})
            return

        try:
            body = self.rfile.read(content_length).decode("utf-8", errors="replace")
            payload = json.loads(body)
        except (json.JSONDecodeError, OSError) as exc:
            self._respond_json(400, {"error": str(exc)})
            return

        alerts = payload.get("alerts", [])
        if not alerts:
            self._respond_json(200, {"processed": 0})
            return

        processed: list[str] = []
        for alert in alerts:
            if alert.get("status") == "resolved":
                fingerprint = alert.get("fingerprint", "")
                # resolved alert は該当 incident を解決
                for inc in self.registry.firing_incidents():
                    if fingerprint and fingerprint in inc.incident_id:
                        self.registry.resolve(inc.incident_id, "alert resolved")
                        self.notifier.notify_resolved(inc.incident_id, "auto-resolved")
                continue

            labels = alert.get("labels", {})
            alertname = labels.get("alertname", "unknown_alert")
            incident_id = _make_incident_id(alertname, labels)

            # 既存インシデントの重複スキップ
            existing = self.registry.get(incident_id)
            if existing and existing.state == IncidentState.FIRING:
                logger.debug("duplicate alert for incident %s, skipping", incident_id)
                continue

            sc = self.classifier.classify(labels)
            runbook_ref = _lookup_runbook(sc.name)

            incident = Incident(
                incident_id=incident_id,
                signal_class=sc.name,
                alertname=alertname,
                alert_labels=labels,
                severity=sc.severity_default,
                runbook_ref=runbook_ref,
            )
            self.registry.register(incident)
            processed.append(incident_id)

            # phase_1_detect: on-call page
            self.notifier.page_oncall(
                incident_id=incident_id,
                signal_class=sc.name,
                alertname=alertname,
                runbook_ref=runbook_ref,
                severity=sc.severity_default,
                tier=1,
            )
            # phase_2_triage: triage channel notification
            self.notifier.notify_triage_open(
                incident_id=incident_id,
                signal_class=sc.name,
                alertname=alertname,
            )
            logger.info("new incident %s signal=%s severity=%s", incident_id, sc.name, sc.severity_default)

        self._respond_json(200, {"processed": len(processed), "incident_ids": processed})

    def _respond_json(self, status: int, data: dict[str, Any]) -> None:
        body = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def _make_handler_class(
    registry: IncidentRegistry,
    notifier: MattermostNotifier,
    classifier: SignalClassifier,
) -> type[WebhookHandler]:
    """クロージャで registry / notifier / classifier を inject したハンドラクラスを返す。"""
    class _Handler(WebhookHandler):
        pass
    _Handler.registry = registry
    _Handler.notifier = notifier
    _Handler.classifier = classifier
    return _Handler


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def run(
    port: int = 9093,
    dry_run: bool = False,
    poll_interval: float = 10.0,
) -> None:
    """escalation_engine を起動する。Ctrl-C で停止。"""
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
    )

    classifier = SignalClassifier()
    notifier = MattermostNotifier(dry_run=dry_run)
    registry = IncidentRegistry()
    monitor = EscalationMonitor(registry, notifier, classifier, poll_interval=poll_interval)

    handler_cls = _make_handler_class(registry, notifier, classifier)
    server = http.server.HTTPServer(("0.0.0.0", port), handler_cls)

    monitor.start()
    logger.info("escalation_engine listening on :%d (dry_run=%s)", port, dry_run)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        logger.info("shutdown requested")
    finally:
        monitor.stop()
        server.server_close()
        logger.info("escalation_engine stopped")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="self_escalation_engine: Alertmanager webhook → 3-tier escalation DAG"
    )
    parser.add_argument("--port", type=int, default=int(os.environ.get("ESCALATION_PORT", 9093)))
    parser.add_argument("--dry-run", action="store_true", default=_DRY_RUN_DEFAULT)
    parser.add_argument("--poll-interval", type=float, default=10.0)
    args = parser.parse_args(argv)
    run(port=args.port, dry_run=args.dry_run, poll_interval=args.poll_interval)
    return 0


_DRY_RUN_DEFAULT = os.environ.get("ESCALATION_DRY_RUN", "false").lower() in ("1", "true", "yes")

if __name__ == "__main__":
    sys.exit(main())
