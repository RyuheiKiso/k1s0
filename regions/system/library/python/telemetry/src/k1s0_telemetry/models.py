"""テレメトリー設定モデル"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class LogConfig:
    """ログ設定。"""

    level: str = "INFO"
    format: str = "json"  # "json" or "text"


@dataclass
class TraceConfig:
    """分散トレーシング設定。"""

    enabled: bool = False
    endpoint: str = ""
    sample_rate: float = 1.0
    service_name: str = ""


@dataclass
class MetricsConfig:
    """メトリクス設定。"""

    enabled: bool = True
    path: str = "/metrics"


@dataclass
class TelemetryConfig:
    """テレメトリー全体設定。"""

    service_name: str
    service_version: str = "0.1.0"
    log: LogConfig = field(default_factory=LogConfig)
    trace: TraceConfig = field(default_factory=TraceConfig)
    metrics: MetricsConfig = field(default_factory=MetricsConfig)
