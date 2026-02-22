namespace K1s0.System.Outbox;

public interface IOutboxStore
{
    Task SaveAsync(OutboxMessage message, CancellationToken ct = default);

    Task<IReadOnlyList<OutboxMessage>> FetchPendingAsync(int limit = 100, CancellationToken ct = default);

    Task MarkPublishedAsync(Guid id, CancellationToken ct = default);

    Task MarkFailedAsync(Guid id, string error, CancellationToken ct = default);
}
