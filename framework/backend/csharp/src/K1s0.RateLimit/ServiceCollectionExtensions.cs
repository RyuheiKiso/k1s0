using Microsoft.Extensions.DependencyInjection;

namespace K1s0.RateLimit;

/// <summary>
/// Extension methods for registering K1s0.RateLimit services with the dependency injection container.
/// </summary>
public static class ServiceCollectionExtensions
{
    /// <summary>
    /// Adds a token bucket rate limiter to the specified <see cref="IServiceCollection"/>.
    /// </summary>
    /// <param name="services">The service collection to add services to.</param>
    /// <param name="capacity">The maximum number of tokens the bucket can hold.</param>
    /// <param name="refillRatePerSecond">The number of tokens added per second.</param>
    /// <returns>The same service collection for chaining.</returns>
    public static IServiceCollection AddTokenBucketRateLimit(this IServiceCollection services, long capacity, double refillRatePerSecond)
    {
        services.AddSingleton<IRateLimiter>(new TokenBucketLimiter(capacity, refillRatePerSecond));
        return services;
    }

    /// <summary>
    /// Adds a sliding window rate limiter to the specified <see cref="IServiceCollection"/>.
    /// </summary>
    /// <param name="services">The service collection to add services to.</param>
    /// <param name="windowSize">The size of the sliding time window.</param>
    /// <param name="maxRequests">The maximum number of requests allowed within the window.</param>
    /// <returns>The same service collection for chaining.</returns>
    public static IServiceCollection AddSlidingWindowRateLimit(this IServiceCollection services, TimeSpan windowSize, long maxRequests)
    {
        services.AddSingleton<IRateLimiter>(new SlidingWindowLimiter(windowSize, maxRequests));
        return services;
    }
}
