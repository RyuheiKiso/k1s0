using System.Net;
using K1s0.Error;

namespace K1s0.RateLimit;

/// <summary>
/// Exception thrown when a rate limit has been exceeded.
/// Carries the retry-after duration so callers can determine when to retry.
/// </summary>
public class RateLimitExceededException : K1s0Exception
{
    /// <summary>
    /// Gets the duration after which the request can be retried.
    /// </summary>
    public TimeSpan RetryAfter { get; }

    /// <summary>
    /// Initializes a new instance of the <see cref="RateLimitExceededException"/> class.
    /// </summary>
    /// <param name="retryAfter">The duration after which the request can be retried.</param>
    public RateLimitExceededException(TimeSpan retryAfter)
        : base("ratelimit.exceeded", $"Rate limit exceeded. Retry after {retryAfter.TotalSeconds:F0}s.", HttpStatusCode.TooManyRequests)
    {
        RetryAfter = retryAfter;
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="RateLimitExceededException"/> class with an inner exception.
    /// </summary>
    /// <param name="retryAfter">The duration after which the request can be retried.</param>
    /// <param name="innerException">The inner exception.</param>
    public RateLimitExceededException(TimeSpan retryAfter, Exception innerException)
        : base("ratelimit.exceeded", $"Rate limit exceeded. Retry after {retryAfter.TotalSeconds:F0}s.", HttpStatusCode.TooManyRequests, innerException)
    {
        RetryAfter = retryAfter;
    }
}
