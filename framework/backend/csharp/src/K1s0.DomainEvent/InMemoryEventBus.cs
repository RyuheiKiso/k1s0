using System.Collections.Concurrent;

namespace K1s0.DomainEvent;

/// <summary>
/// An in-memory event bus that implements both <see cref="IEventPublisher"/> and <see cref="IEventSubscriber"/>.
/// Suitable for single-process scenarios and testing.
/// </summary>
public sealed class InMemoryEventBus : IEventPublisher, IEventSubscriber
{
    private readonly ConcurrentDictionary<string, List<IEventHandler>> _handlers = new();
    private readonly object _lock = new();

    /// <inheritdoc />
    public Task PublishAsync(EventEnvelope envelope, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(envelope);
        return PublishCoreAsync(envelope, cancellationToken);
    }

    /// <inheritdoc />
    public async Task PublishBatchAsync(IEnumerable<EventEnvelope> envelopes, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(envelopes);

        foreach (var envelope in envelopes)
        {
            cancellationToken.ThrowIfCancellationRequested();
            await PublishCoreAsync(envelope, cancellationToken).ConfigureAwait(false);
        }
    }

    /// <inheritdoc />
    public Task<IDisposable> SubscribeAsync(IEventHandler handler, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(handler);

        lock (_lock)
        {
            var list = _handlers.GetOrAdd(handler.EventType, _ => []);
            list.Add(handler);
        }

        IDisposable subscription = new Subscription(this, handler);
        return Task.FromResult(subscription);
    }

    private async Task PublishCoreAsync(EventEnvelope envelope, CancellationToken cancellationToken)
    {
        List<IEventHandler> snapshot;

        lock (_lock)
        {
            if (!_handlers.TryGetValue(envelope.EventType, out var list))
            {
                return;
            }

            snapshot = [.. list];
        }

        foreach (var handler in snapshot)
        {
            cancellationToken.ThrowIfCancellationRequested();
            await handler.HandleAsync(envelope, cancellationToken).ConfigureAwait(false);
        }
    }

    private void Unsubscribe(IEventHandler handler)
    {
        lock (_lock)
        {
            if (_handlers.TryGetValue(handler.EventType, out var list))
            {
                list.Remove(handler);
            }
        }
    }

    private sealed class Subscription(InMemoryEventBus bus, IEventHandler handler) : IDisposable
    {
        private int _disposed;

        public void Dispose()
        {
            if (Interlocked.Exchange(ref _disposed, 1) == 0)
            {
                bus.Unsubscribe(handler);
            }
        }
    }
}
