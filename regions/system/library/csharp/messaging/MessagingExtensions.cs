using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.Messaging;

public static class MessagingExtensions
{
    public static IServiceCollection AddK1s0Messaging(
        this IServiceCollection services,
        MessagingConfig config)
    {
        services.AddSingleton(config);
        services.AddSingleton<IEventProducer>(sp => new KafkaEventProducer(config));
        return services;
    }

    public static IServiceCollection AddK1s0Messaging(
        this IServiceCollection services,
        MessagingConfig config,
        params string[] topics)
    {
        services.AddSingleton(config);
        services.AddSingleton<IEventProducer>(sp => new KafkaEventProducer(config));
        services.AddSingleton<IEventConsumer>(sp => new KafkaEventConsumer(config, topics));
        return services;
    }
}
