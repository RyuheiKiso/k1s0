from abc import ABC, abstractmethod
from datetime import datetime, timezone
import time

from k1s0_migration.config import MigrationConfig
from k1s0_migration.model import (
    MigrationReport,
    MigrationStatus,
    PendingMigration,
    compute_checksum,
)


class MigrationRunner(ABC):
    @abstractmethod
    async def run_up(self) -> MigrationReport: ...

    @abstractmethod
    async def run_down(self, steps: int) -> MigrationReport: ...

    @abstractmethod
    async def status(self) -> list[MigrationStatus]: ...

    @abstractmethod
    async def pending(self) -> list[PendingMigration]: ...


class InMemoryMigrationRunner(MigrationRunner):
    def __init__(
        self,
        config: MigrationConfig,
        ups: list[tuple[str, str, str]],
        downs: list[tuple[str, str, str]],
    ) -> None:
        self._config = config
        self._up_migrations = sorted(
            [{"version": v, "name": n, "content": c} for v, n, c in ups],
            key=lambda x: x["version"],
        )
        self._down_migrations = {v: {"version": v, "name": n, "content": c} for v, n, c in downs}
        self._applied: list[MigrationStatus] = []

    async def run_up(self) -> MigrationReport:
        start = time.monotonic()
        applied_versions = {s.version for s in self._applied}
        count = 0

        for mf in self._up_migrations:
            if mf["version"] in applied_versions:
                continue
            cs = compute_checksum(mf["content"])
            self._applied.append(
                MigrationStatus(
                    version=mf["version"],
                    name=mf["name"],
                    applied_at=datetime.now(timezone.utc),
                    checksum=cs,
                )
            )
            count += 1

        elapsed = time.monotonic() - start
        return MigrationReport(applied_count=count, elapsed=elapsed)

    async def run_down(self, steps: int) -> MigrationReport:
        start = time.monotonic()
        count = 0

        for _ in range(steps):
            if not self._applied:
                break
            self._applied.pop()
            count += 1

        elapsed = time.monotonic() - start
        return MigrationReport(applied_count=count, elapsed=elapsed)

    async def status(self) -> list[MigrationStatus]:
        applied_map = {s.version: s for s in self._applied}
        result = []

        for mf in self._up_migrations:
            cs = compute_checksum(mf["content"])
            applied = applied_map.get(mf["version"])
            result.append(
                MigrationStatus(
                    version=mf["version"],
                    name=mf["name"],
                    applied_at=applied.applied_at if applied else None,
                    checksum=cs,
                )
            )

        return result

    async def pending(self) -> list[PendingMigration]:
        applied_versions = {s.version for s in self._applied}
        return [
            PendingMigration(version=mf["version"], name=mf["name"])
            for mf in self._up_migrations
            if mf["version"] not in applied_versions
        ]
