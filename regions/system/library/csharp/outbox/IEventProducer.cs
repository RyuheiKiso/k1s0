namespace K1s0.System.Outbox;

public interface IEventProducer
{
    Task PublishAsync(OutboxMessage message, CancellationToken ct = default);
}
