namespace K1s0.System.Messaging;

public sealed class NoOpEventProducer : IEventProducer
{
    public Task PublishAsync(EventEnvelope envelope, CancellationToken ct = default)
    {
        return Task.CompletedTask;
    }

    public Task PublishBatchAsync(IReadOnlyList<EventEnvelope> envelopes, CancellationToken ct = default)
    {
        return Task.CompletedTask;
    }

    public ValueTask DisposeAsync()
    {
        return ValueTask.CompletedTask;
    }
}
