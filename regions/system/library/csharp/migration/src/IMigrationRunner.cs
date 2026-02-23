namespace K1s0.System.Migration;

public interface IMigrationRunner
{
    Task<MigrationReport> RunUpAsync(CancellationToken ct = default);

    Task<MigrationReport> RunDownAsync(int steps, CancellationToken ct = default);

    Task<IReadOnlyList<MigrationStatus>> StatusAsync(CancellationToken ct = default);

    Task<IReadOnlyList<PendingMigration>> PendingAsync(CancellationToken ct = default);
}
