"""Schema Registry データモデル"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


class SchemaType(StrEnum):
    """スキーマタイプ。"""

    AVRO = "AVRO"
    JSON = "JSON"
    PROTOBUF = "PROTOBUF"


class CompatibilityMode(StrEnum):
    """互換性モード。"""

    BACKWARD = "BACKWARD"
    BACKWARD_TRANSITIVE = "BACKWARD_TRANSITIVE"
    FORWARD = "FORWARD"
    FORWARD_TRANSITIVE = "FORWARD_TRANSITIVE"
    FULL = "FULL"
    FULL_TRANSITIVE = "FULL_TRANSITIVE"
    NONE = "NONE"


@dataclass
class RegisteredSchema:
    """登録済みスキーマ情報。"""

    id: int
    subject: str
    version: int
    schema: str
    schema_type: SchemaType = SchemaType.AVRO


@dataclass
class SchemaRegistryConfig:
    """Schema Registry 接続設定。"""

    url: str
    username: str = ""
    password: str = ""
    compatibility_mode: CompatibilityMode = CompatibilityMode.BACKWARD
    timeout_seconds: float = 10.0
