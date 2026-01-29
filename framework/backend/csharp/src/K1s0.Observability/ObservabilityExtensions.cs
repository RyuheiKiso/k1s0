using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using OpenTelemetry.Metrics;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;

namespace K1s0.Observability;

/// <summary>
/// Extension methods for setting up k1s0 observability (tracing, metrics, logging).
/// </summary>
public static class ObservabilityExtensions
{
    /// <summary>
    /// Adds k1s0 observability services including OpenTelemetry tracing and metrics.
    /// Reads the service name from configuration key "app:name" (default: "k1s0-service").
    /// Reads the OTLP endpoint from "observability:otlp_endpoint" (default: "http://localhost:4317").
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <param name="configuration">The application configuration.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0Observability(
        this IServiceCollection services,
        IConfiguration configuration)
    {
        ArgumentNullException.ThrowIfNull(services);
        ArgumentNullException.ThrowIfNull(configuration);

        string serviceName = configuration["app:name"] ?? "k1s0-service";
        string otlpEndpoint = configuration["observability:otlp_endpoint"] ?? "http://localhost:4317";

        services.AddSingleton<K1s0Metrics>();

        services.AddOpenTelemetry()
            .ConfigureResource(resource => resource.AddService(serviceName))
            .WithTracing(tracing =>
            {
                tracing
                    .AddSource(K1s0ActivitySource.Name)
                    .AddOtlpExporter(options => options.Endpoint = new Uri(otlpEndpoint));
            })
            .WithMetrics(metrics =>
            {
                metrics
                    .AddMeter(K1s0Metrics.MeterName)
                    .AddOtlpExporter(options => options.Endpoint = new Uri(otlpEndpoint));
            });

        return services;
    }
}
