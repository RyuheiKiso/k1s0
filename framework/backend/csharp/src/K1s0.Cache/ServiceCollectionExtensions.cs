using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Cache;

/// <summary>
/// Extension methods for registering K1s0 cache services with dependency injection.
/// </summary>
public static class ServiceCollectionExtensions
{
    /// <summary>
    /// Registers K1s0 cache services including <see cref="ConnectionManager"/>,
    /// <see cref="CacheClient"/>, <see cref="ICacheOperations"/>, and <see cref="CacheMetrics"/>.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <param name="config">The cache configuration.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0Cache(this IServiceCollection services, CacheConfig config)
    {
        ArgumentNullException.ThrowIfNull(services);
        ArgumentNullException.ThrowIfNull(config);

        services.AddSingleton(config);
        services.AddSingleton<ConnectionManager>();
        services.AddSingleton<CacheClient>();
        services.AddSingleton<ICacheOperations>(sp => sp.GetRequiredService<CacheClient>());
        services.AddSingleton<IHashOperations>(sp => sp.GetRequiredService<CacheClient>());
        services.AddSingleton<IListOperations>(sp => sp.GetRequiredService<CacheClient>());
        services.AddSingleton<ISetOperations>(sp => sp.GetRequiredService<CacheClient>());
        services.AddSingleton<CacheMetrics>();
        services.AddSingleton<CacheHealthChecker>();
        return services;
    }
}
