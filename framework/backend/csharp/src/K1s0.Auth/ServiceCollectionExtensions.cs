using K1s0.Auth.Jwt;
using Microsoft.Extensions.DependencyInjection;

namespace K1s0.Auth;

/// <summary>
/// Extension methods for registering K1s0 authentication services.
/// </summary>
public static class ServiceCollectionExtensions
{
    /// <summary>
    /// Adds K1s0 authentication services to the service collection.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <param name="config">The JWT verifier configuration.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0Auth(this IServiceCollection services, JwtVerifierConfig config)
    {
        services.AddSingleton(config);
        services.AddSingleton<JwtVerifier>();
        services.AddSingleton<InMemoryBlacklist>();
        services.AddSingleton<ITokenBlacklist>(sp => sp.GetRequiredService<InMemoryBlacklist>());
        services.AddSingleton<AuditLogger>();
        return services;
    }
}
