namespace K1s0.Cache;

/// <summary>
/// Thread-safe cache metrics tracking for hit/miss rates and operation counts.
/// </summary>
public sealed class CacheMetrics
{
    private long _hitCount;
    private long _missCount;
    private long _operationCount;

    /// <summary>
    /// Gets the total number of cache hits.
    /// </summary>
    public long HitCount => Interlocked.Read(ref _hitCount);

    /// <summary>
    /// Gets the total number of cache misses.
    /// </summary>
    public long MissCount => Interlocked.Read(ref _missCount);

    /// <summary>
    /// Gets the total number of cache operations.
    /// </summary>
    public long OperationCount => Interlocked.Read(ref _operationCount);

    /// <summary>
    /// Gets the cache hit rate as a value between 0.0 and 1.0.
    /// Returns 0.0 if no hits or misses have been recorded.
    /// </summary>
    public double HitRate
    {
        get
        {
            long hits = HitCount;
            long misses = MissCount;
            long total = hits + misses;
            return total == 0 ? 0.0 : (double)hits / total;
        }
    }

    /// <summary>
    /// Records a cache hit.
    /// </summary>
    public void RecordHit()
    {
        Interlocked.Increment(ref _hitCount);
        Interlocked.Increment(ref _operationCount);
    }

    /// <summary>
    /// Records a cache miss.
    /// </summary>
    public void RecordMiss()
    {
        Interlocked.Increment(ref _missCount);
        Interlocked.Increment(ref _operationCount);
    }
}
