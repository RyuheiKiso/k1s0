namespace K1s0.DomainEvent;

/// <summary>
/// Publishes domain events to subscribers.
/// </summary>
public interface IEventPublisher
{
    /// <summary>
    /// Publishes a single event envelope.
    /// </summary>
    /// <param name="envelope">The event envelope to publish.</param>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    Task PublishAsync(EventEnvelope envelope, CancellationToken cancellationToken = default);

    /// <summary>
    /// Publishes a batch of event envelopes.
    /// </summary>
    /// <param name="envelopes">The event envelopes to publish.</param>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    Task PublishBatchAsync(IEnumerable<EventEnvelope> envelopes, CancellationToken cancellationToken = default);
}
