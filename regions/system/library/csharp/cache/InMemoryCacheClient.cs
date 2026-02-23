namespace K1s0.System.Cache;

public class InMemoryCacheClient : ICacheClient
{
    private record Entry(string Value, DateTime? ExpiresAt)
    {
        public bool IsExpired => ExpiresAt.HasValue && ExpiresAt.Value <= DateTime.UtcNow;
    }

    private readonly Dictionary<string, Entry> _store = new();
    private readonly SemaphoreSlim _lock = new(1, 1);

    public async Task<string?> GetAsync(string key)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_store.TryGetValue(key, out var entry))
            {
                if (entry.IsExpired)
                {
                    _store.Remove(key);
                    return null;
                }

                return entry.Value;
            }

            return null;
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task SetAsync(string key, string value, TimeSpan? ttl = null)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            var expiresAt = ttl.HasValue ? DateTime.UtcNow + ttl.Value : (DateTime?)null;
            _store[key] = new Entry(value, expiresAt);
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

    public async Task<bool> ExistsAsync(string key)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_store.TryGetValue(key, out var entry))
            {
                if (entry.IsExpired)
                {
                    _store.Remove(key);
                    return false;
                }

                return true;
            }

            return false;
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task<bool> SetNxAsync(string key, string value, TimeSpan ttl)
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_store.TryGetValue(key, out var existing) && !existing.IsExpired)
            {
                return false;
            }

            _store[key] = new Entry(value, DateTime.UtcNow + ttl);
            return true;
        }
        finally
        {
            _lock.Release();
        }
    }
}
