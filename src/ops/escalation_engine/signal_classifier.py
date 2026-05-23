"""src/ops/escalation_engine/signal_classifier.py

Alertmanager alert の labels から signal_class を判定するロジック。
仕様: 17_運用ループ適合仕様.md §軸 class

signal_class は signal_classes.yaml が SoT。alert labels の優先マッチング順で判定する。
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

try:
    import yaml
    _HAS_YAML = True
except ImportError:
    _HAS_YAML = False


@dataclass(frozen=True)
class SignalClass:
    name: str
    severity_default: str       # v1_page | v1_notify | v1_freeze
    paging_budget_weight: float
    postmortem_class: str
    min_action_count_v1: int
    ack_window_seconds: int     # severity_default から導出


_SEVERITY_TO_ACK_WINDOW: dict[str, int] = {
    "v1_freeze": 180,   # 3 分
    "v1_page": 300,     # 5 分
    "v1_notify": 3600,  # 60 分
}

# signal_class 名 → ラベルマッチパターン（Alertmanager alert labels を想定）
_LABEL_MATCH_PATTERNS: dict[str, list[str]] = {
    "v1_slo_breach": [
        r"alertname.*slo.*breach",
        r"alertname.*burn.*rate",
        r"alertname.*error.*budget",
        r"ops_signal_class=v1_slo_breach",
    ],
    "v1_capacity_breach": [
        r"alertname.*capacity.*breach",
        r"alertname.*quota.*exceeded",
        r"alertname.*headroom.*low",
        r"ops_signal_class=v1_capacity_breach",
    ],
    "v1_change_induced": [
        r"alertname.*canary.*breach",
        r"alertname.*rollback.*trigger",
        r"alertname.*migration.*slo",
        r"alertname.*topology.*drift",
        r"ops_signal_class=v1_change_induced",
    ],
    "v1_dependency_outage": [
        r"alertname.*dependency.*outage",
        r"alertname.*idp.*unavailable",
        r"alertname.*upstream.*down",
        r"alertname.*dns.*failure",
        r"ops_signal_class=v1_dependency_outage",
    ],
    "v1_drill_failure": [
        r"alertname.*drill.*failure",
        r"alertname.*chaos.*unexpected",
        r"alertname.*restore.*drill.*failed",
        r"ops_signal_class=v1_drill_failure",
    ],
}


def _load_signal_classes(classes_yaml: Path) -> dict[str, SignalClass]:
    """signal_classes.yaml から SignalClass dict を構築する。"""
    if not _HAS_YAML or not classes_yaml.exists():
        return _builtin_signal_classes()

    data = yaml.safe_load(classes_yaml.read_text(encoding="utf-8")) or {}
    result: dict[str, SignalClass] = {}
    for entry in data.get("signal_classes", []):
        name = str(entry.get("signal_class", ""))
        severity = str(entry.get("severity_default", "v1_page"))
        ack_window = _SEVERITY_TO_ACK_WINDOW.get(severity, 300)
        result[name] = SignalClass(
            name=name,
            severity_default=severity,
            paging_budget_weight=float(entry.get("paging_budget_weight", 1.0)),
            postmortem_class=str(entry.get("postmortem_class", "v1_full")),
            min_action_count_v1=int(entry.get("min_action_count_v1", 20)),
            ack_window_seconds=ack_window,
        )
    return result if result else _builtin_signal_classes()


def _builtin_signal_classes() -> dict[str, SignalClass]:
    """signal_classes.yaml がない場合のフォールバック定義。"""
    return {
        "v1_slo_breach": SignalClass("v1_slo_breach", "v1_page", 1.0, "v1_full", 30, 300),
        "v1_capacity_breach": SignalClass("v1_capacity_breach", "v1_page", 0.8, "v1_full", 25, 300),
        "v1_change_induced": SignalClass("v1_change_induced", "v1_page", 1.2, "v1_full", 30, 300),
        "v1_dependency_outage": SignalClass("v1_dependency_outage", "v1_page", 0.7, "v1_full", 25, 300),
        "v1_drill_failure": SignalClass("v1_drill_failure", "v1_notify", 0.3, "v1_lightweight", 20, 3600),
    }


class SignalClassifier:
    """Alertmanager alert から signal_class を判定するクラス。

    判定順序:
    1. labels["ops_signal_class"] が明示されていれば最優先
    2. alertname / labels を正規表現でマッチ
    3. マッチなしは v1_slo_breach にフォールバック（保守的側）
    """

    def __init__(self, classes_yaml: Path | None = None) -> None:
        _default = Path(__file__).parent.parent / "ops_loop" / "signal_classes.yaml"
        self._classes = _load_signal_classes(classes_yaml or _default)

    def classify(self, alert_labels: dict[str, Any]) -> SignalClass:
        """alert_labels から signal_class を返す。"""
        # 明示ラベル優先
        explicit = str(alert_labels.get("ops_signal_class", "")).strip()
        if explicit in self._classes:
            return self._classes[explicit]

        # alertname + labels を結合してパターンマッチ
        label_text = " ".join(
            f"{k}={v}" for k, v in alert_labels.items()
        ).lower()

        for class_name, patterns in _LABEL_MATCH_PATTERNS.items():
            for pattern in patterns:
                if re.search(pattern, label_text, re.IGNORECASE):
                    return self._classes.get(class_name, self._default_class())

        # フォールバック: v1_slo_breach（最も保守的な対処を選ぶ）
        return self._default_class()

    def _default_class(self) -> SignalClass:
        return self._classes.get(
            "v1_slo_breach",
            SignalClass("v1_slo_breach", "v1_page", 1.0, "v1_full", 30, 300),
        )

    def get(self, class_name: str) -> SignalClass | None:
        return self._classes.get(class_name)

    def all_classes(self) -> list[SignalClass]:
        return list(self._classes.values())
