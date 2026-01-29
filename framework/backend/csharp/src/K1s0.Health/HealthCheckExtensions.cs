using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Diagnostics.HealthChecks;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Diagnostics.HealthChecks;

namespace K1s0.Health;

/// <summary>
/// Extension methods for configuring k1s0 health checks.
/// </summary>
public static class HealthCheckExtensions
{
    /// <summary>
    /// Adds k1s0 health check services including liveness and readiness probes.
    /// </summary>
    /// <param name="services">The service collection.</param>
    /// <returns>The service collection for chaining.</returns>
    public static IServiceCollection AddK1s0HealthChecks(this IServiceCollection services)
    {
        ArgumentNullException.ThrowIfNull(services);

        services.AddSingleton<ReadinessCheck>();

        services.AddHealthChecks()
            .AddCheck<LivenessCheck>("liveness", tags: ["live"])
            .AddCheck<ReadinessCheck>("readiness", tags: ["ready"]);

        return services;
    }

    /// <summary>
    /// Maps k1s0 health check endpoints:
    /// <list type="bullet">
    /// <item><description>/healthz/live - Liveness probe</description></item>
    /// <item><description>/healthz/ready - Readiness probe</description></item>
    /// </list>
    /// </summary>
    /// <param name="app">The web application.</param>
    /// <returns>The web application for chaining.</returns>
    public static WebApplication MapK1s0HealthChecks(this WebApplication app)
    {
        ArgumentNullException.ThrowIfNull(app);

        app.MapHealthChecks("/healthz/live", new HealthCheckOptions
        {
            Predicate = check => check.Tags.Contains("live"),
            ResultStatusCodes =
            {
                [HealthStatus.Healthy] = StatusCodes.Status200OK,
                [HealthStatus.Degraded] = StatusCodes.Status200OK,
                [HealthStatus.Unhealthy] = StatusCodes.Status503ServiceUnavailable,
            },
        });

        app.MapHealthChecks("/healthz/ready", new HealthCheckOptions
        {
            Predicate = check => check.Tags.Contains("ready"),
            ResultStatusCodes =
            {
                [HealthStatus.Healthy] = StatusCodes.Status200OK,
                [HealthStatus.Degraded] = StatusCodes.Status200OK,
                [HealthStatus.Unhealthy] = StatusCodes.Status503ServiceUnavailable,
            },
        });

        return app;
    }
}
