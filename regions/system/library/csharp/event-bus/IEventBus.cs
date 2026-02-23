namespace K1s0.System.EventBus;

public record Event(
    string Id,
    string EventType,
    Dictionary<string, object> Payload,
    DateTimeOffset Timestamp);

public delegate Task EventHandler(Event @event, CancellationToken ct = default);

public interface IEventBus
{
    void Subscribe(string eventType, EventHandler handler);

    void Unsubscribe(string eventType);

    Task PublishAsync(Event @event, CancellationToken ct = default);
}
