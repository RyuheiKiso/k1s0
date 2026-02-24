namespace K1s0.System.EventStore;

public sealed class InMemorySnapshotStore : ISnapshotStore
{
    private readonly Dictionary<string, Snapshot> _snapshots = new();

    public Task SaveSnapshotAsync(Snapshot snapshot, CancellationToken ct = default)
    {
        _snapshots[snapshot.StreamId] = snapshot;
        return Task.CompletedTask;
    }

    public Task<Snapshot?> LoadSnapshotAsync(string streamId, CancellationToken ct = default)
    {
        _snapshots.TryGetValue(streamId, out var snapshot);
        return Task.FromResult(snapshot);
    }
}
