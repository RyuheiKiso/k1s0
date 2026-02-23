from dataclasses import dataclass, field
from pathlib import Path


@dataclass(frozen=True)
class MigrationConfig:
    migrations_dir: Path
    database_url: str
    table_name: str = field(default="_migrations")
