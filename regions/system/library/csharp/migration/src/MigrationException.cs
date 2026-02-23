namespace K1s0.System.Migration;

public class MigrationException : Exception
{
    public string? ErrorCode { get; }

    public MigrationException(string message, string? errorCode = null)
        : base(message)
    {
        ErrorCode = errorCode;
    }

    public MigrationException(string message, Exception innerException, string? errorCode = null)
        : base(message, innerException)
    {
        ErrorCode = errorCode;
    }

    public static MigrationException ConnectionFailed(string message, Exception? inner = null) =>
        inner is not null
            ? new MigrationException(message, inner, "CONNECTION_FAILED")
            : new MigrationException(message, "CONNECTION_FAILED");

    public static MigrationException MigrationFailed(string version, string message, Exception? inner = null) =>
        inner is not null
            ? new MigrationException($"Migration {version} failed: {message}", inner, "MIGRATION_FAILED")
            : new MigrationException($"Migration {version} failed: {message}", "MIGRATION_FAILED");

    public static MigrationException ChecksumMismatch(string version, string expected, string actual) =>
        new($"Checksum mismatch for version {version}: expected {expected}, actual {actual}", "CHECKSUM_MISMATCH");

    public static MigrationException DirectoryNotFound(string path) =>
        new($"Directory not found: {path}", "DIRECTORY_NOT_FOUND");
}
