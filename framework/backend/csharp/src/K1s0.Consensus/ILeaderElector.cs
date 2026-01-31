namespace K1s0.Consensus;

/// <summary>
/// Interface for leader election using lease-based coordination.
/// </summary>
public interface ILeaderElector
{
    /// <summary>
    /// Attempts to acquire the leader lease for the given key.
    /// Returns the lease if acquired, or <c>null</c> if another node holds it.
    /// </summary>
    /// <param name="leaseKey">The leadership group key.</param>
    /// <param name="holderId">The unique identifier of this node.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The acquired lease, or <c>null</c> if not acquired.</returns>
    Task<LeaderLease?> TryAcquireAsync(string leaseKey, string holderId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Renews an existing lease. Fails if the lease has expired or is held by another node.
    /// </summary>
    /// <param name="lease">The lease to renew.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The renewed lease with an updated expiration and fence token.</returns>
    /// <exception cref="LeaseExpiredException">Thrown when the lease has expired.</exception>
    Task<LeaderLease> RenewAsync(LeaderLease lease, CancellationToken cancellationToken = default);

    /// <summary>
    /// Voluntarily releases the leader lease.
    /// </summary>
    /// <param name="lease">The lease to release.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    Task ReleaseAsync(LeaderLease lease, CancellationToken cancellationToken = default);

    /// <summary>
    /// Returns the current leader lease for the given key, if any.
    /// </summary>
    /// <param name="leaseKey">The leadership group key.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The current lease, or <c>null</c> if no leader.</returns>
    Task<LeaderLease?> CurrentLeaderAsync(string leaseKey, CancellationToken cancellationToken = default);

    /// <summary>
    /// Returns an async stream of leader events for the given key.
    /// The stream emits events when leadership changes, renewals occur, or leadership is lost.
    /// </summary>
    /// <param name="leaseKey">The leadership group key.</param>
    /// <param name="holderId">The unique identifier of this node.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>An async enumerable of leader events.</returns>
    IAsyncEnumerable<LeaderEvent> WatchAsync(string leaseKey, string holderId, CancellationToken cancellationToken = default);
}
