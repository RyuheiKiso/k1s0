namespace K1s0.System.Messaging;

public interface IEventConsumer : IAsyncDisposable
{
    Task<ConsumedMessage> ReceiveAsync(CancellationToken ct = default);

    Task CommitAsync(ConsumedMessage message, CancellationToken ct = default);
}
