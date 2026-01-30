namespace K1s0.DomainEvent.Outbox;

/// <summary>
/// Persistence abstraction for the outbox pattern.
/// </summary>
public interface IOutboxStore
{
    /// <summary>
    /// Inserts a new outbox entry.
    /// </summary>
    /// <param name="entry">The outbox entry to insert.</param>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    Task InsertAsync(OutboxEntry entry, CancellationToken cancellationToken = default);

    /// <summary>
    /// Fetches pending outbox entries up to the specified limit.
    /// </summary>
    /// <param name="limit">Maximum number of entries to fetch.</param>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    /// <returns>A list of pending outbox entries.</returns>
    Task<IReadOnlyList<OutboxEntry>> FetchPendingAsync(int limit, CancellationToken cancellationToken = default);

    /// <summary>
    /// Marks an outbox entry as successfully published.
    /// </summary>
    /// <param name="id">The identifier of the entry to mark.</param>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    Task MarkPublishedAsync(Guid id, CancellationToken cancellationToken = default);

    /// <summary>
    /// Marks an outbox entry as failed, incrementing its retry count.
    /// </summary>
    /// <param name="id">The identifier of the entry to mark.</param>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    Task MarkFailedAsync(Guid id, CancellationToken cancellationToken = default);
}
