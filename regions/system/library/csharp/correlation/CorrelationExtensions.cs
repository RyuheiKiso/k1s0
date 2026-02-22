using Microsoft.Extensions.DependencyInjection;

namespace K1s0.System.Correlation;

public static class CorrelationExtensions
{
    public static IServiceCollection AddK1s0Correlation(
        this IServiceCollection services)
    {
        services.AddScoped(_ => CorrelationContext.New());
        return services;
    }
}
