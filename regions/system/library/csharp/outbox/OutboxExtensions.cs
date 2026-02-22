using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.Outbox;

public static class OutboxExtensions
{
    public static IServiceCollection AddK1s0Outbox(
        this IServiceCollection services,
        OutboxConfig config)
    {
        ArgumentNullException.ThrowIfNull(config);

        services.AddSingleton(config);
        services.AddSingleton<IOutboxStore>(new PostgresOutboxStore(config.ConnectionString));
        services.AddHostedService<OutboxProcessor>();
        return services;
    }
}
