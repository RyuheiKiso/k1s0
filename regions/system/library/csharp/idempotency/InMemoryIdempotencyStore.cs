namespace K1s0.System.Idempotency;

public class InMemoryIdempotencyStore : IIdempotencyStore
{
    private readonly Dictionary<string, IdempotencyRecord> _store = new();
    private readonly SemaphoreSlim _lock = new(1, 1);

    public async Task<IdempotencyRecord?> GetAsync(string key)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_store.TryGetValue(key, out var record))
            {
                if (record.IsExpired())
                {
                    _store.Remove(key);
                    return null;
                }

                return record;
            }

            return null;
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task InsertAsync(IdempotencyRecord record)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_store.ContainsKey(record.Key))
            {
                throw new DuplicateKeyException(record.Key);
            }

            var withTimestamp = record with { CreatedAt = DateTimeOffset.UtcNow };
            _store[record.Key] = withTimestamp;
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task UpdateAsync(string key, IdempotencyStatus status, string? body = null, int? code = null)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_store.TryGetValue(key, out var record))
            {
                throw new KeyNotFoundException($"Key not found: {key}");
            }

            _store[key] = record with
            {
                Status = status,
                ResponseBody = body,
                StatusCode = code,
                CompletedAt = DateTimeOffset.UtcNow,
            };
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task<bool> DeleteAsync(string key)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            return _store.Remove(key);
        }
        finally
        {
            _lock.Release();
        }
    }
}
