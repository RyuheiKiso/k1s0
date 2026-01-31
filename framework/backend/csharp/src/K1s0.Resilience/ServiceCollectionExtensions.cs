using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Resilience;

/// <summary>
/// Extension methods for registering K1s0.Resilience services with the dependency injection container.
/// </summary>
public static class ServiceCollectionExtensions
{
    /// <summary>
    /// Adds K1s0 resilience services to the specified <see cref="IServiceCollection"/>.
    /// </summary>
    /// <param name="services">The service collection to add services to.</param>
    /// <returns>The same service collection for chaining.</returns>
    public static IServiceCollection AddK1s0Resilience(this IServiceCollection services)
    {
        return services;
    }
}
