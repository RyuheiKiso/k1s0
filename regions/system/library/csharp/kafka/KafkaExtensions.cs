using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.Kafka;

public static class KafkaExtensions
{
    public static IServiceCollection AddK1s0Kafka(this IServiceCollection services, KafkaConfig config)
    {
        services.AddSingleton(config);
        services.AddSingleton<IKafkaHealthCheck, KafkaHealthCheck>();
        return services;
    }
}
