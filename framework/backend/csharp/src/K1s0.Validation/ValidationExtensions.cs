using System.Reflection;
using FluentValidation;
using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Validation;

/// <summary>
/// Extension methods for registering k1s0 validators with dependency injection.
/// </summary>
public static class ValidationExtensions
{
    /// <summary>
    /// Registers all FluentValidation validators found in the specified assemblies.
    /// If no assemblies are provided, the calling assembly is scanned.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <param name="assemblies">Assemblies to scan for validators. Defaults to the calling assembly.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0Validation(
        this IServiceCollection services,
        params Assembly[] assemblies)
    {
        ArgumentNullException.ThrowIfNull(services);

        if (assemblies.Length == 0)
        {
            assemblies = [Assembly.GetCallingAssembly()];
        }

        services.AddValidatorsFromAssemblies(assemblies, ServiceLifetime.Scoped);

        return services;
    }
}
