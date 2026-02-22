using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.Auth;

public static class AuthExtensions
{
    public static IServiceCollection AddK1s0Auth(this IServiceCollection services, AuthConfig config)
    {
        services.AddSingleton(config);
        services.AddHttpClient(nameof(HttpJwksFetcher));
        services.AddSingleton<IJwksFetcher, HttpJwksFetcher>();
        services.AddSingleton<IJwksVerifier, JwksVerifier>();
        return services;
    }
}
