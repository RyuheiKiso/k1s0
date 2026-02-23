from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Optional
import hashlib


class MigrationDirection(Enum):
    UP = "up"
    DOWN = "down"


@dataclass(frozen=True)
class MigrationReport:
    applied_count: int
    elapsed: float
    errors: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class MigrationStatus:
    version: str
    name: str
    applied_at: Optional[datetime]
    checksum: str


@dataclass(frozen=True)
class PendingMigration:
    version: str
    name: str


def parse_filename(
    filename: str,
) -> Optional[tuple[str, str, MigrationDirection]]:
    if not filename.endswith(".sql"):
        return None

    stem = filename[:-4]

    if stem.endswith(".up"):
        direction = MigrationDirection.UP
        rest = stem[:-3]
    elif stem.endswith(".down"):
        direction = MigrationDirection.DOWN
        rest = stem[:-5]
    else:
        return None

    idx = rest.find("_")
    if idx <= 0 or idx >= len(rest) - 1:
        return None

    version = rest[:idx]
    name = rest[idx + 1 :]

    return (version, name, direction)


def compute_checksum(content: str) -> str:
    return hashlib.sha256(content.encode("utf-8")).hexdigest()
