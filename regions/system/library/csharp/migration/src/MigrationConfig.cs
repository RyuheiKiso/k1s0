namespace K1s0.System.Migration;

public record MigrationConfig(
    string MigrationsDir,
    string DatabaseUrl,
    string TableName = "_migrations");
