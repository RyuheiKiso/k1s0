namespace K1s0.RateLimit;

/// <summary>
/// Defines the contract for rate limiting strategies.
/// Implementations must be thread-safe.
/// </summary>
public interface IRateLimiter
{
    /// <summary>
    /// Attempts to acquire a single token from the rate limiter.
    /// </summary>
    /// <param name="cancellationToken">A token to observe while waiting for the task to complete.</param>
    /// <returns><c>true</c> if the token was acquired; <c>false</c> if the rate limit has been exceeded.</returns>
    Task<bool> TryAcquireAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets the estimated time until the next token becomes available.
    /// </summary>
    /// <returns>The duration until a token is available, or <see cref="TimeSpan.Zero"/> if tokens are currently available.</returns>
    TimeSpan TimeUntilAvailable();

    /// <summary>
    /// Gets the number of tokens currently available for consumption.
    /// </summary>
    /// <returns>The number of available tokens.</returns>
    long AvailableTokens();

    /// <summary>
    /// Gets the current rate limiting statistics.
    /// </summary>
    /// <returns>A snapshot of the current statistics.</returns>
    RateLimitStats GetStats();
}

/// <summary>
/// Represents a snapshot of rate limiter statistics.
/// </summary>
/// <param name="Allowed">The total number of allowed requests.</param>
/// <param name="Rejected">The total number of rejected requests.</param>
/// <param name="Total">The total number of requests (allowed + rejected).</param>
/// <param name="Available">The number of currently available tokens.</param>
public record RateLimitStats(long Allowed, long Rejected, long Total, long Available);
