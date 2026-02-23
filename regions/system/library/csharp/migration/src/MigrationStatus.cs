namespace K1s0.System.Migration;

public record MigrationStatus(
    string Version,
    string Name,
    DateTimeOffset? AppliedAt,
    string Checksum);
