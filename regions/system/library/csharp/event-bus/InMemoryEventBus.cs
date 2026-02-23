namespace K1s0.System.EventBus;

public class InMemoryEventBus : IEventBus
{
    private readonly Dictionary<string, List<EventHandler>> _handlers = new();
    private readonly object _lock = new();

    public void Subscribe(string eventType, EventHandler handler)
    {
        lock (_lock)
        {
            if (!_handlers.ContainsKey(eventType))
            {
                _handlers[eventType] = new List<EventHandler>();
            }

            _handlers[eventType].Add(handler);
        }
    }

    public void Unsubscribe(string eventType)
    {
        lock (_lock)
        {
            _handlers.Remove(eventType);
        }
    }

    public async Task PublishAsync(Event @event, CancellationToken ct = default)
    {
        List<EventHandler> handlers;
        lock (_lock)
        {
            if (!_handlers.TryGetValue(@event.EventType, out var list))
            {
                return;
            }

            handlers = list.ToList();
        }

        foreach (var handler in handlers)
        {
            await handler(@event, ct).ConfigureAwait(false);
        }
    }
}
