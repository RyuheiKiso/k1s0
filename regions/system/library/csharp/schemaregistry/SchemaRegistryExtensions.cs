using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.SchemaRegistry;

public static class SchemaRegistryExtensions
{
    public static IServiceCollection AddK1s0SchemaRegistry(this IServiceCollection services, SchemaRegistryConfig config)
    {
        services.AddSingleton(config);
        services.AddSingleton<ISchemaRegistryClient>(_ => new ConfluentSchemaRegistryClient(config));
        return services;
    }
}
