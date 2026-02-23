namespace K1s0.System.DistributedLock;

public class InMemoryDistributedLock : IDistributedLock
{
    private readonly Dictionary<string, (string Token, DateTime ExpiresAt)> _locks = new();
    private readonly SemaphoreSlim _sem = new(1, 1);

    public async Task<LockGuard> AcquireAsync(string key, TimeSpan ttl, CancellationToken ct = default)
    {
        await _sem.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            if (_locks.TryGetValue(key, out var existing) && existing.ExpiresAt > DateTime.UtcNow)
            {
                throw new LockException($"Lock already held: {key}");
            }

            var token = Guid.NewGuid().ToString();
            _locks[key] = (token, DateTime.UtcNow + ttl);
            return new LockGuard(key, token);
        }
        finally
        {
            _sem.Release();
        }
    }

    public async Task ReleaseAsync(LockGuard guard, CancellationToken ct = default)
    {
        await _sem.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            if (_locks.TryGetValue(guard.Key, out var existing) && existing.Token == guard.Token)
            {
                _locks.Remove(guard.Key);
            }
            else
            {
                throw new LockException($"Lock not owned: {guard.Key}");
            }
        }
        finally
        {
            _sem.Release();
        }
    }

    public async Task<bool> IsLockedAsync(string key, CancellationToken ct = default)
    {
        await _sem.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            if (_locks.TryGetValue(key, out var existing))
            {
                if (existing.ExpiresAt <= DateTime.UtcNow)
                {
                    _locks.Remove(key);
                    return false;
                }

                return true;
            }

            return false;
        }
        finally
        {
            _sem.Release();
        }
    }
}
