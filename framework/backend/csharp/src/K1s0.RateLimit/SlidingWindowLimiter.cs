namespace K1s0.RateLimit;

/// <summary>
/// Implements the sliding window rate limiting algorithm.
/// Tracks request timestamps and allows up to a maximum number of requests within a sliding time window.
/// Thread-safe for concurrent usage.
/// </summary>
public sealed class SlidingWindowLimiter : IRateLimiter
{
    private readonly TimeSpan _windowSize;
    private readonly long _maxRequests;
    private readonly Queue<DateTime> _timestamps = new();
    private readonly object _lock = new();
    private long _allowed;
    private long _rejected;

    /// <summary>
    /// Initializes a new instance of the <see cref="SlidingWindowLimiter"/> class.
    /// </summary>
    /// <param name="windowSize">The size of the sliding time window.</param>
    /// <param name="maxRequests">The maximum number of requests allowed within the window.</param>
    /// <exception cref="ArgumentOutOfRangeException">Thrown when windowSize is not positive or maxRequests is not positive.</exception>
    public SlidingWindowLimiter(TimeSpan windowSize, long maxRequests)
    {
        if (windowSize <= TimeSpan.Zero)
        {
            throw new ArgumentOutOfRangeException(nameof(windowSize), "Window size must be positive.");
        }

        if (maxRequests <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(maxRequests), "Max requests must be positive.");
        }

        _windowSize = windowSize;
        _maxRequests = maxRequests;
    }

    /// <inheritdoc />
    public Task<bool> TryAcquireAsync(CancellationToken cancellationToken = default)
    {
        lock (_lock)
        {
            EvictExpired();

            if (_timestamps.Count < _maxRequests)
            {
                _timestamps.Enqueue(DateTime.UtcNow);
                Interlocked.Increment(ref _allowed);
                return Task.FromResult(true);
            }

            Interlocked.Increment(ref _rejected);
            return Task.FromResult(false);
        }
    }

    /// <inheritdoc />
    public TimeSpan TimeUntilAvailable()
    {
        lock (_lock)
        {
            EvictExpired();

            if (_timestamps.Count < _maxRequests)
            {
                return TimeSpan.Zero;
            }

            if (_timestamps.TryPeek(out var oldest))
            {
                var expiry = oldest + _windowSize;
                var remaining = expiry - DateTime.UtcNow;
                return remaining > TimeSpan.Zero ? remaining : TimeSpan.Zero;
            }

            return TimeSpan.Zero;
        }
    }

    /// <inheritdoc />
    public long AvailableTokens()
    {
        lock (_lock)
        {
            EvictExpired();
            return Math.Max(0, _maxRequests - _timestamps.Count);
        }
    }

    /// <inheritdoc />
    public RateLimitStats GetStats()
    {
        lock (_lock)
        {
            EvictExpired();
            var allowed = Interlocked.Read(ref _allowed);
            var rejected = Interlocked.Read(ref _rejected);
            var available = Math.Max(0, _maxRequests - _timestamps.Count);
            return new RateLimitStats(allowed, rejected, allowed + rejected, available);
        }
    }

    private void EvictExpired()
    {
        var cutoff = DateTime.UtcNow - _windowSize;
        while (_timestamps.TryPeek(out var oldest) && oldest <= cutoff)
        {
            _timestamps.Dequeue();
        }
    }
}
