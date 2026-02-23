namespace K1s0.System.EventStore;

public interface IEventStore
{
    Task<long> AppendAsync(string streamId, IReadOnlyList<EventEnvelope> events, long? expectedVersion = null);

    Task<IReadOnlyList<EventEnvelope>> LoadAsync(string streamId);

    Task<IReadOnlyList<EventEnvelope>> LoadFromAsync(string streamId, long fromVersion);

    Task<bool> ExistsAsync(string streamId);

    Task<long> CurrentVersionAsync(string streamId);
}
