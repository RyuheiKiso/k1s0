namespace K1s0.System.DistributedLock;

public record LockGuard(string Key, string Token);

public class LockException : Exception
{
    public LockException(string msg)
        : base(msg)
    {
    }
}

public interface IDistributedLock
{
    Task<LockGuard> AcquireAsync(string key, TimeSpan ttl, CancellationToken ct = default);

    Task ReleaseAsync(LockGuard guard, CancellationToken ct = default);

    Task<bool> IsLockedAsync(string key, CancellationToken ct = default);
}
