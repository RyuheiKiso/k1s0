namespace K1s0.Consensus;

/// <summary>
/// An RAII-style guard that releases a distributed lock when disposed.
/// Implements <see cref="IAsyncDisposable"/> for use with <c>await using</c>.
/// </summary>
public sealed class LockGuard : IAsyncDisposable
{
    private readonly IDistributedLock _lock;
    private readonly string _lockKey;
    private readonly string _holderId;
    private int _disposed;

    /// <summary>
    /// The lock key held by this guard.
    /// </summary>
    public string LockKey => _lockKey;

    /// <summary>
    /// The holder identity for this guard.
    /// </summary>
    public string HolderId => _holderId;

    /// <summary>
    /// Creates a new <see cref="LockGuard"/>.
    /// </summary>
    /// <param name="distributedLock">The lock implementation to call for release.</param>
    /// <param name="lockKey">The lock key.</param>
    /// <param name="holderId">The holder identity.</param>
    public LockGuard(IDistributedLock distributedLock, string lockKey, string holderId)
    {
        _lock = distributedLock;
        _lockKey = lockKey;
        _holderId = holderId;
    }

    /// <summary>
    /// Extends the lock expiration.
    /// </summary>
    /// <param name="extension">How long to extend.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns><c>true</c> if extended successfully.</returns>
    public Task<bool> ExtendAsync(TimeSpan? extension = null, CancellationToken cancellationToken = default) =>
        _lock.ExtendAsync(_lockKey, _holderId, extension, cancellationToken);

    /// <inheritdoc />
    public async ValueTask DisposeAsync()
    {
        if (Interlocked.CompareExchange(ref _disposed, 1, 0) == 0)
        {
            await _lock.UnlockAsync(_lockKey, _holderId).ConfigureAwait(false);
        }
    }
}
