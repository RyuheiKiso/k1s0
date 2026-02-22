using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.ServiceAuth;

public static class ServiceAuthExtensions
{
    public static IServiceCollection AddK1s0ServiceAuth(
        this IServiceCollection services,
        ServiceAuthConfig config)
    {
        services.AddSingleton(config);
        services.AddSingleton<IServiceAuthClient>(sp =>
        {
            var httpClient = new HttpClient();
            return new ServiceAuthClient(httpClient, config);
        });
        return services;
    }
}
