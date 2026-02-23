import pytest
from pathlib import Path

from k1s0_migration import (
    InMemoryMigrationRunner,
    MigrationConfig,
    MigrationDirection,
    parse_filename,
    compute_checksum,
    MigrationError,
    ConnectionFailedError,
    ChecksumMismatchError,
    DirectoryNotFoundError,
)


def create_runner() -> InMemoryMigrationRunner:
    config = MigrationConfig(
        migrations_dir=Path("."),
        database_url="memory://",
    )
    ups = [
        ("20240101000001", "create_users", "CREATE TABLE users (id INT);"),
        ("20240101000002", "add_email", "ALTER TABLE users ADD COLUMN email TEXT;"),
        ("20240201000001", "create_orders", "CREATE TABLE orders (id INT);"),
    ]
    downs = [
        ("20240101000001", "create_users", "DROP TABLE users;"),
        ("20240101000002", "add_email", "ALTER TABLE users DROP COLUMN email;"),
        ("20240201000001", "create_orders", "DROP TABLE orders;"),
    ]
    return InMemoryMigrationRunner(config, ups, downs)


class TestParseFilename:
    def test_parse_up_migration(self) -> None:
        result = parse_filename("20240101000001_create_users.up.sql")
        assert result is not None
        version, name, direction = result
        assert version == "20240101000001"
        assert name == "create_users"
        assert direction == MigrationDirection.UP

    def test_parse_down_migration(self) -> None:
        result = parse_filename("20240101000001_create_users.down.sql")
        assert result is not None
        version, name, direction = result
        assert version == "20240101000001"
        assert name == "create_users"
        assert direction == MigrationDirection.DOWN

    def test_parse_invalid_filenames(self) -> None:
        assert parse_filename("invalid.sql") is None
        assert parse_filename("no_direction.sql") is None
        assert parse_filename("_.up.sql") is None


class TestChecksum:
    def test_deterministic(self) -> None:
        content = "CREATE TABLE users (id SERIAL PRIMARY KEY);"
        assert compute_checksum(content) == compute_checksum(content)

    def test_differs_for_different_content(self) -> None:
        assert compute_checksum("CREATE TABLE users;") != compute_checksum(
            "CREATE TABLE orders;"
        )


class TestExceptions:
    def test_migration_error(self) -> None:
        err = MigrationError("test", code="TEST")
        assert str(err) == "test"
        assert err.code == "TEST"

    def test_connection_failed(self) -> None:
        err = ConnectionFailedError("fail")
        assert err.code == "CONNECTION_FAILED"

    def test_checksum_mismatch(self) -> None:
        err = ChecksumMismatchError("v1", "abc", "def")
        assert err.code == "CHECKSUM_MISMATCH"
        assert err.version == "v1"

    def test_directory_not_found(self) -> None:
        err = DirectoryNotFoundError("/tmp")
        assert err.code == "DIRECTORY_NOT_FOUND"
        assert err.path == "/tmp"


class TestMigrationConfig:
    def test_default_table_name(self) -> None:
        config = MigrationConfig(
            migrations_dir=Path("."), database_url="memory://"
        )
        assert config.table_name == "_migrations"

    def test_custom_table_name(self) -> None:
        config = MigrationConfig(
            migrations_dir=Path("."),
            database_url="memory://",
            table_name="custom",
        )
        assert config.table_name == "custom"


class TestInMemoryMigrationRunner:
    @pytest.fixture
    def runner(self) -> InMemoryMigrationRunner:
        return create_runner()

    async def test_run_up_applies_all(self, runner: InMemoryMigrationRunner) -> None:
        report = await runner.run_up()
        assert report.applied_count == 3
        assert len(report.errors) == 0

    async def test_run_up_idempotent(self, runner: InMemoryMigrationRunner) -> None:
        await runner.run_up()
        report = await runner.run_up()
        assert report.applied_count == 0

    async def test_run_down_one_step(self, runner: InMemoryMigrationRunner) -> None:
        await runner.run_up()
        report = await runner.run_down(1)
        assert report.applied_count == 1

        pending = await runner.pending()
        assert len(pending) == 1
        assert pending[0].version == "20240201000001"

    async def test_run_down_multiple_steps(
        self, runner: InMemoryMigrationRunner
    ) -> None:
        await runner.run_up()
        report = await runner.run_down(2)
        assert report.applied_count == 2

        pending = await runner.pending()
        assert len(pending) == 2

    async def test_run_down_more_than_applied(
        self, runner: InMemoryMigrationRunner
    ) -> None:
        await runner.run_up()
        report = await runner.run_down(10)
        assert report.applied_count == 3

    async def test_status_all_pending(
        self, runner: InMemoryMigrationRunner
    ) -> None:
        statuses = await runner.status()
        assert len(statuses) == 3
        for s in statuses:
            assert s.applied_at is None

    async def test_status_after_apply(
        self, runner: InMemoryMigrationRunner
    ) -> None:
        await runner.run_up()
        statuses = await runner.status()
        assert len(statuses) == 3
        for s in statuses:
            assert s.applied_at is not None

    async def test_pending_returns_unapplied(
        self, runner: InMemoryMigrationRunner
    ) -> None:
        pending = await runner.pending()
        assert len(pending) == 3
        assert pending[0].version == "20240101000001"
        assert pending[1].version == "20240101000002"
        assert pending[2].version == "20240201000001"

    async def test_pending_empty_after_apply(
        self, runner: InMemoryMigrationRunner
    ) -> None:
        await runner.run_up()
        pending = await runner.pending()
        assert len(pending) == 0
