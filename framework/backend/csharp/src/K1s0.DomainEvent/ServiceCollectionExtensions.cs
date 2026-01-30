using Microsoft.Extensions.DependencyInjection;

namespace K1s0.DomainEvent;

/// <summary>
/// Extension methods for registering k1s0 domain event services.
/// </summary>
public static class ServiceCollectionExtensions
{
    /// <summary>
    /// Registers the in-memory event bus as both <see cref="IEventPublisher"/> and <see cref="IEventSubscriber"/>.
    /// </summary>
    /// <param name="services">The service collection to add to.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0DomainEvent(this IServiceCollection services)
    {
        services.AddSingleton<InMemoryEventBus>();
        services.AddSingleton<IEventPublisher>(sp => sp.GetRequiredService<InMemoryEventBus>());
        services.AddSingleton<IEventSubscriber>(sp => sp.GetRequiredService<InMemoryEventBus>());
        return services;
    }
}
