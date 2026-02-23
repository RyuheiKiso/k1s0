class MigrationError(Exception):
    def __init__(self, message: str, code: str | None = None) -> None:
        super().__init__(message)
        self.code = code


class ConnectionFailedError(MigrationError):
    def __init__(self, message: str) -> None:
        super().__init__(message, code="CONNECTION_FAILED")


class MigrationFailedError(MigrationError):
    def __init__(self, version: str, message: str) -> None:
        super().__init__(f"Migration {version} failed: {message}", code="MIGRATION_FAILED")
        self.version = version


class ChecksumMismatchError(MigrationError):
    def __init__(self, version: str, expected: str, actual: str) -> None:
        super().__init__(
            f"Checksum mismatch for version {version}: expected {expected}, actual {actual}",
            code="CHECKSUM_MISMATCH",
        )
        self.version = version
        self.expected = expected
        self.actual = actual


class DirectoryNotFoundError(MigrationError):
    def __init__(self, path: str) -> None:
        super().__init__(f"Directory not found: {path}", code="DIRECTORY_NOT_FOUND")
        self.path = path
