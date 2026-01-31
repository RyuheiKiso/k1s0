namespace K1s0.DomainEvent;

/// <summary>
/// Handles domain events of a specific type.
/// </summary>
public interface IEventHandler
{
    /// <summary>
    /// Gets the event type this handler processes.
    /// </summary>
    string EventType { get; }

    /// <summary>
    /// Handles the given event envelope.
    /// </summary>
    /// <param name="envelope">The event envelope to handle.</param>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    Task HandleAsync(EventEnvelope envelope, CancellationToken cancellationToken = default);
}
