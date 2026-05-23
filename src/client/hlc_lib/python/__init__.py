# __init__.py — k1s0-hlc Python パッケージの公開 API を re-export する
from .hlc import (
    HLCTimestamp,
    HLCClock,
    EPOCH,
    before,
    concurrent,
    pack_b64,
    unpack_b64,
)
# hlc_vector モジュールから追加のクラスを re-export する
from .hlc_vector import (
    VectorClock,
    merge_vector,
    HLCSerializer,
    HLCDeserializer,
    HLCIntervalCheck,
    IntervalCheckResult,
    DriftGuard,
    DriftGuardViolation,
    HLCSequencer,
    SequencedTimestamp,
    HLCEventLog,
    HLCLogEntry,
)
# パッケージバージョンを定義する
__version__ = "1.0.0"
# 公開シンボルを明示的に列挙する（全エクスポートを列挙することで IDE 補完を有効化する）
__all__ = [
    "HLCTimestamp",
    "HLCClock",
    "EPOCH",
    "before",
    "concurrent",
    "pack_b64",
    "unpack_b64",
    "VectorClock",
    "merge_vector",
    "HLCSerializer",
    "HLCDeserializer",
    "HLCIntervalCheck",
    "IntervalCheckResult",
    "DriftGuard",
    "DriftGuardViolation",
    "HLCSequencer",
    "SequencedTimestamp",
    "HLCEventLog",
    "HLCLogEntry",
]
