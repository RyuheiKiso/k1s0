"""event_store データモデル"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone


@dataclass
class EventEnvelope:
    """イベントエンベロープ。"""

    event_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    stream_id: str = ""
    version: int = 0
    event_type: str = ""
    payload: dict = field(default_factory=dict)
    metadata: dict = field(default_factory=dict)
    recorded_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
