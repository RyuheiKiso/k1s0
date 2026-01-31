namespace K1s0.Consensus;

/// <summary>
/// Interface for distributed lock operations.
/// </summary>
public interface IDistributedLock
{
    /// <summary>
    /// Attempts to acquire a lock without waiting. Returns <c>null</c> if the lock is already held.
    /// </summary>
    /// <param name="lockKey">The lock identifier.</param>
    /// <param name="holderId">The unique identifier of this node.</param>
    /// <param name="expiration">How long the lock should be held before auto-expiring. If <c>null</c>, uses default.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="LockGuard"/> if acquired, or <c>null</c>.</returns>
    Task<LockGuard?> TryLockAsync(string lockKey, string holderId, TimeSpan? expiration = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// Acquires a lock, waiting up to the specified timeout.
    /// </summary>
    /// <param name="lockKey">The lock identifier.</param>
    /// <param name="holderId">The unique identifier of this node.</param>
    /// <param name="expiration">How long the lock should be held before auto-expiring. If <c>null</c>, uses default.</param>
    /// <param name="waitTimeout">Maximum time to wait for the lock. If <c>null</c>, uses default.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="LockGuard"/> when the lock is acquired.</returns>
    /// <exception cref="LockAcquisitionException">Thrown when the lock cannot be acquired within the timeout.</exception>
    Task<LockGuard> LockAsync(string lockKey, string holderId, TimeSpan? expiration = null, TimeSpan? waitTimeout = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// Extends the expiration of an existing lock.
    /// </summary>
    /// <param name="lockKey">The lock identifier.</param>
    /// <param name="holderId">The unique identifier of this node.</param>
    /// <param name="extension">How long to extend. If <c>null</c>, uses the default expiration.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns><c>true</c> if the lock was extended, <c>false</c> if not held.</returns>
    Task<bool> ExtendAsync(string lockKey, string holderId, TimeSpan? extension = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// Releases a lock.
    /// </summary>
    /// <param name="lockKey">The lock identifier.</param>
    /// <param name="holderId">The unique identifier of this node.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns><c>true</c> if the lock was released, <c>false</c> if not held.</returns>
    Task<bool> UnlockAsync(string lockKey, string holderId, CancellationToken cancellationToken = default);
}
