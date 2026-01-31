namespace K1s0.RateLimit;

/// <summary>
/// Implements the token bucket rate limiting algorithm.
/// Tokens are added at a fixed rate up to a maximum capacity.
/// Thread-safe for concurrent usage.
/// </summary>
public sealed class TokenBucketLimiter : IRateLimiter
{
    private readonly long _capacity;
    private readonly double _refillRatePerSecond;
    private readonly object _lock = new();
    private double _tokens;
    private long _lastRefillTicks;
    private long _allowed;
    private long _rejected;

    /// <summary>
    /// Initializes a new instance of the <see cref="TokenBucketLimiter"/> class.
    /// </summary>
    /// <param name="capacity">The maximum number of tokens the bucket can hold.</param>
    /// <param name="refillRatePerSecond">The number of tokens added per second.</param>
    /// <exception cref="ArgumentOutOfRangeException">Thrown when capacity or refillRatePerSecond is not positive.</exception>
    public TokenBucketLimiter(long capacity, double refillRatePerSecond)
    {
        if (capacity <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(capacity), "Capacity must be positive.");
        }

        if (refillRatePerSecond <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(refillRatePerSecond), "Refill rate must be positive.");
        }

        _capacity = capacity;
        _refillRatePerSecond = refillRatePerSecond;
        _tokens = capacity;
        _lastRefillTicks = DateTime.UtcNow.Ticks;
    }

    /// <inheritdoc />
    public Task<bool> TryAcquireAsync(CancellationToken cancellationToken = default)
    {
        lock (_lock)
        {
            Refill();

            if (_tokens >= 1.0)
            {
                _tokens -= 1.0;
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
            Refill();

            if (_tokens >= 1.0)
            {
                return TimeSpan.Zero;
            }

            var tokensNeeded = 1.0 - _tokens;
            var secondsUntilAvailable = tokensNeeded / _refillRatePerSecond;
            return TimeSpan.FromSeconds(secondsUntilAvailable);
        }
    }

    /// <inheritdoc />
    public long AvailableTokens()
    {
        lock (_lock)
        {
            Refill();
            return (long)_tokens;
        }
    }

    /// <inheritdoc />
    public RateLimitStats GetStats()
    {
        lock (_lock)
        {
            Refill();
            var allowed = Interlocked.Read(ref _allowed);
            var rejected = Interlocked.Read(ref _rejected);
            return new RateLimitStats(allowed, rejected, allowed + rejected, (long)_tokens);
        }
    }

    private void Refill()
    {
        var now = DateTime.UtcNow.Ticks;
        var elapsed = TimeSpan.FromTicks(now - _lastRefillTicks);
        var newTokens = elapsed.TotalSeconds * _refillRatePerSecond;

        if (newTokens > 0)
        {
            _tokens = Math.Min(_capacity, _tokens + newTokens);
            _lastRefillTicks = now;
        }
    }
}
