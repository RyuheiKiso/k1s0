namespace K1s0.System.EventStore;

public interface ISnapshotStore
{
    Task SaveSnapshotAsync(Snapshot snapshot, CancellationToken ct = default);

    Task<Snapshot?> LoadSnapshotAsync(string streamId, CancellationToken ct = default);
}
