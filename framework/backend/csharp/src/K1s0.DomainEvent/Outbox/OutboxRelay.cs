namespace K1s0.DomainEvent.Outbox;

/// <summary>
/// Polls the outbox store for pending entries and publishes them via the event publisher.
/// Failed entries are retried up to <see cref="MaxRetries"/> times before being marked as failed.
/// </summary>
public sealed class OutboxRelay
{
    private readonly IOutboxStore _store;
    private readonly IEventPublisher _publisher;

    /// <summary>
    /// Gets or sets the interval between polling cycles.
    /// </summary>
    public TimeSpan PollInterval { get; set; } = TimeSpan.FromSeconds(5);

    /// <summary>
    /// Gets or sets the maximum number of retry attempts before marking an entry as failed.
    /// </summary>
    public int MaxRetries { get; set; } = 3;

    /// <summary>
    /// Gets or sets the maximum number of pending entries to fetch per polling cycle.
    /// </summary>
    public int BatchSize { get; set; } = 100;

    /// <summary>
    /// Initializes a new instance of the <see cref="OutboxRelay"/> class.
    /// </summary>
    /// <param name="store">The outbox store to poll for pending entries.</param>
    /// <param name="publisher">The event publisher to publish entries through.</param>
    public OutboxRelay(IOutboxStore store, IEventPublisher publisher)
    {
        ArgumentNullException.ThrowIfNull(store);
        ArgumentNullException.ThrowIfNull(publisher);

        _store = store;
        _publisher = publisher;
    }

    /// <summary>
    /// Runs the relay loop, polling the outbox store and publishing pending entries until cancellation is requested.
    /// </summary>
    /// <param name="cancellationToken">A token to signal shutdown.</param>
    public async Task RunAsync(CancellationToken cancellationToken)
    {
        while (!cancellationToken.IsCancellationRequested)
        {
            await RelayPendingAsync(cancellationToken).ConfigureAwait(false);

            try
            {
                await Task.Delay(PollInterval, cancellationToken).ConfigureAwait(false);
            }
            catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
            {
                break;
            }
        }
    }

    /// <summary>
    /// Performs a single relay cycle: fetches pending entries and attempts to publish them.
    /// </summary>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    public async Task RelayPendingAsync(CancellationToken cancellationToken = default)
    {
        var entries = await _store.FetchPendingAsync(BatchSize, cancellationToken).ConfigureAwait(false);

        foreach (var entry in entries)
        {
            cancellationToken.ThrowIfCancellationRequested();

            try
            {
                var envelope = System.Text.Json.JsonSerializer.Deserialize<EventEnvelope>(entry.Payload)
                    ?? throw new InvalidOperationException($"Failed to deserialize outbox entry {entry.Id}.");

                await _publisher.PublishAsync(envelope, cancellationToken).ConfigureAwait(false);
                await _store.MarkPublishedAsync(entry.Id, cancellationToken).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch
            {
                if (entry.RetryCount + 1 >= MaxRetries)
                {
                    await _store.MarkFailedAsync(entry.Id, cancellationToken).ConfigureAwait(false);
                }
                else
                {
                    await _store.MarkFailedAsync(entry.Id, cancellationToken).ConfigureAwait(false);
                }
            }
        }
    }
}
