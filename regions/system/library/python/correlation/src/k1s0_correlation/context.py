"""CorrelationContext データクラス定義"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field


@dataclass
class CorrelationContext:
    """相関IDとトレースIDを保持するコンテキスト。"""

    correlation_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    trace_id: str = field(default_factory=lambda: uuid.uuid4().hex)
    request_id: str | None = None

    def __post_init__(self) -> None:
        if not self.correlation_id:
            raise ValueError("correlation_id cannot be empty")
        if not self.trace_id:
            raise ValueError("trace_id cannot be empty")
