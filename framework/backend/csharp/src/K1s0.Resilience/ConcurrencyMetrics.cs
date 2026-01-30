namespace K1s0.Resilience;

/// <summary>
/// Thread-safe metrics for concurrency limiter usage.
/// </summary>
public class ConcurrencyMetrics
{
    private int _activeCount;
    private long _rejectedCount;

    /// <summary>
    /// Gets the current number of active concurrent executions.
    /// </summary>
    public int ActiveCount => Volatile.Read(ref _activeCount);

    /// <summary>
    /// Gets the total number of rejected executions due to concurrency limits.
    /// </summary>
    public long RejectedCount => Interlocked.Read(ref _rejectedCount);

    /// <summary>
    /// Increments the active execution count.
    /// </summary>
    internal void IncrementActive() => Interlocked.Increment(ref _activeCount);

    /// <summary>
    /// Decrements the active execution count.
    /// </summary>
    internal void DecrementActive() => Interlocked.Decrement(ref _activeCount);

    /// <summary>
    /// Increments the rejected execution count.
    /// </summary>
    internal void IncrementRejected() => Interlocked.Increment(ref _rejectedCount);
}
