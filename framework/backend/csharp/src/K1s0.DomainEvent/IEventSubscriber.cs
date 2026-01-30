namespace K1s0.DomainEvent;

/// <summary>
/// Subscribes event handlers to receive domain events.
/// </summary>
public interface IEventSubscriber
{
    /// <summary>
    /// Subscribes an event handler. Returns an <see cref="IDisposable"/> that removes the subscription when disposed.
    /// </summary>
    /// <param name="handler">The event handler to subscribe.</param>
    /// <param name="cancellationToken">A token to cancel the operation.</param>
    /// <returns>A disposable subscription that unsubscribes the handler when disposed.</returns>
    Task<IDisposable> SubscribeAsync(IEventHandler handler, CancellationToken cancellationToken = default);
}
