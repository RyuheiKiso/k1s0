namespace K1s0.System.Messaging;

public interface IEventProducer : IAsyncDisposable
{
    Task PublishAsync(EventEnvelope envelope, CancellationToken ct = default);

    Task PublishBatchAsync(IReadOnlyList<EventEnvelope> envelopes, CancellationToken ct = default);
}
