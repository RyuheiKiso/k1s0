using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Consensus;

/// <summary>
/// Extension methods for registering k1s0 consensus services.
/// </summary>
public static class ServiceCollectionExtensions
{
    /// <summary>
    /// Registers all consensus services with their default implementations.
    /// Uses PostgreSQL for leader election and locks, and registers the saga orchestrator.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0Consensus(this IServiceCollection services)
    {
        services.AddDbLeaderElector();
        services.AddDbDistributedLock();
        services.AddSagaOrchestrator();
        return services;
    }

    /// <summary>
    /// Registers the PostgreSQL-backed leader elector as <see cref="ILeaderElector"/>.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddDbLeaderElector(this IServiceCollection services)
    {
        services.AddSingleton<ILeaderElector, DbLeaderElector>();
        return services;
    }

    /// <summary>
    /// Registers the PostgreSQL-backed distributed lock as <see cref="IDistributedLock"/>.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddDbDistributedLock(this IServiceCollection services)
    {
        services.AddSingleton<IDistributedLock, DbDistributedLock>();
        return services;
    }

    /// <summary>
    /// Registers the Redis-backed distributed lock as <see cref="IDistributedLock"/>.
    /// Requires an <see cref="StackExchange.Redis.IConnectionMultiplexer"/> to be registered.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddRedisDistributedLock(this IServiceCollection services)
    {
        services.AddSingleton<IDistributedLock, RedisDistributedLock>();
        return services;
    }

    /// <summary>
    /// Registers the saga orchestrator and choreography timeout monitor.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddSagaOrchestrator(this IServiceCollection services)
    {
        services.AddSingleton<SagaOrchestrator>();
        services.AddHostedService<ChoreographyTimeoutMonitor>();
        return services;
    }
}
