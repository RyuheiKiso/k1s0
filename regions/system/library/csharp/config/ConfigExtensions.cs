using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.Config;

public static class ConfigExtensions
{
    public static IServiceCollection AddK1s0Config(
        this IServiceCollection services,
        string basePath,
        string? envPath = null)
    {
        var config = ConfigLoader.Load(basePath, envPath);
        services.AddSingleton(config);
        return services;
    }
}
