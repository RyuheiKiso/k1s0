namespace K1s0.System.EventStore;

public sealed class InMemoryEventStore : IEventStore
{
    private readonly Dictionary<string, List<EventEnvelope>> _streams = new();
    private readonly SemaphoreSlim _lock = new(1, 1);

    public async Task<long> AppendAsync(string streamId, IReadOnlyList<EventEnvelope> events, long? expectedVersion = null)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_streams.TryGetValue(streamId, out var stream))
            {
                stream = [];
                _streams[streamId] = stream;
            }

            var currentVersion = stream.Count > 0 ? stream[^1].Version : 0;

            if (expectedVersion.HasValue && currentVersion != expectedVersion.Value)
            {
                throw new VersionConflictException(expectedVersion.Value, currentVersion);
            }

            var nextVersion = currentVersion;
            foreach (var evt in events)
            {
                nextVersion++;
                var stamped = evt with
                {
                    Version = nextVersion,
                    RecordedAt = evt.RecordedAt ?? DateTimeOffset.UtcNow,
                };
                stream.Add(stamped);
            }

            return nextVersion;
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task<IReadOnlyList<EventEnvelope>> LoadAsync(string streamId)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_streams.TryGetValue(streamId, out var stream))
            {
                return Array.Empty<EventEnvelope>();
            }

            return stream.ToList();
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task<IReadOnlyList<EventEnvelope>> LoadFromAsync(string streamId, long fromVersion)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_streams.TryGetValue(streamId, out var stream))
            {
                return Array.Empty<EventEnvelope>();
            }

            return stream.Where(e => e.Version >= fromVersion).ToList();
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task<bool> ExistsAsync(string streamId)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            return _streams.ContainsKey(streamId) && _streams[streamId].Count > 0;
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task<long> CurrentVersionAsync(string streamId)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_streams.TryGetValue(streamId, out var stream) || stream.Count == 0)
            {
                return 0;
            }

            return stream[^1].Version;
        }
        finally
        {
            _lock.Release();
        }
    }
}
