namespace K1s0.System.Migration;

public class InMemoryMigrationRunner : IMigrationRunner
{
    private readonly MigrationConfig _config;
    private readonly List<MigrationEntry> _upMigrations;
    private readonly Dictionary<string, MigrationEntry> _downMigrations;
    private readonly List<MigrationStatus> _applied = [];

    public InMemoryMigrationRunner(
        MigrationConfig config,
        IEnumerable<MigrationEntry> ups,
        IEnumerable<MigrationEntry> downs)
    {
        _config = config;
        _upMigrations = ups.OrderBy(u => u.Version).ToList();
        _downMigrations = downs.ToDictionary(d => d.Version);
    }

    public Task<MigrationReport> RunUpAsync(CancellationToken ct = default)
    {
        var sw = global::System.Diagnostics.Stopwatch.StartNew();
        var appliedVersions = _applied.Select(s => s.Version).ToHashSet();
        var count = 0;

        foreach (var mf in _upMigrations)
        {
            ct.ThrowIfCancellationRequested();
            if (appliedVersions.Contains(mf.Version))
            {
                continue;
            }

            _applied.Add(new MigrationStatus(
                mf.Version,
                mf.Name,
                DateTimeOffset.UtcNow,
                MigrationFileParser.ComputeChecksum(mf.Content)));
            count++;
        }

        sw.Stop();
        return Task.FromResult(new MigrationReport(count, sw.Elapsed, Array.Empty<string>()));
    }

    public Task<MigrationReport> RunDownAsync(int steps, CancellationToken ct = default)
    {
        var sw = global::System.Diagnostics.Stopwatch.StartNew();
        var count = 0;

        for (var i = 0; i < steps; i++)
        {
            ct.ThrowIfCancellationRequested();
            if (_applied.Count == 0)
            {
                break;
            }

            _applied.RemoveAt(_applied.Count - 1);
            count++;
        }

        sw.Stop();
        return Task.FromResult(new MigrationReport(count, sw.Elapsed, Array.Empty<string>()));
    }

    public Task<IReadOnlyList<MigrationStatus>> StatusAsync(CancellationToken ct = default)
    {
        var appliedMap = _applied.ToDictionary(s => s.Version);
        var statuses = _upMigrations.Select(mf =>
        {
            var cs = MigrationFileParser.ComputeChecksum(mf.Content);
            appliedMap.TryGetValue(mf.Version, out var applied);
            return new MigrationStatus(mf.Version, mf.Name, applied?.AppliedAt, cs);
        }).ToList();

        return Task.FromResult<IReadOnlyList<MigrationStatus>>(statuses);
    }

    public Task<IReadOnlyList<PendingMigration>> PendingAsync(CancellationToken ct = default)
    {
        var appliedVersions = _applied.Select(s => s.Version).ToHashSet();
        var pending = _upMigrations
            .Where(mf => !appliedVersions.Contains(mf.Version))
            .Select(mf => new PendingMigration(mf.Version, mf.Name))
            .ToList();

        return Task.FromResult<IReadOnlyList<PendingMigration>>(pending);
    }
}

public record MigrationEntry(string Version, string Name, string Content);
