using Microsoft.Extensions.DependencyInjection;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using Serilog;

namespace K1s0.System.Telemetry;

public static class TelemetryExtensions
{
    public static IServiceCollection AddK1s0Telemetry(
        this IServiceCollection services,
        TelemetryConfig config)
    {
        services.AddSingleton(config);

        // Serilog
        Log.Logger = K1s0Logger.NewLogger(config);
        services.AddSingleton(Log.Logger);

        // Metrics
        services.AddSingleton<K1s0Metrics>();

        // Middleware
        services.AddSingleton<Middleware.HttpTelemetryMiddleware>();
        services.AddSingleton<Middleware.GrpcTelemetryInterceptor>();

        // OpenTelemetry Tracing
        if (config.Endpoint is not null)
        {
            services.AddOpenTelemetry()
                .WithTracing(builder =>
                {
                    builder
                        .SetResourceBuilder(ResourceBuilder
                            .CreateDefault()
                            .AddService(
                                serviceName: config.ServiceName,
                                serviceVersion: config.Version))
                        .AddAspNetCoreInstrumentation()
                        .AddGrpcClientInstrumentation()
                        .SetSampler(new TraceIdRatioBasedSampler(config.SampleRate))
                        .AddOtlpExporter(opts =>
                        {
                            opts.Endpoint = new Uri(config.Endpoint);
                        });
                });
        }

        return services;
    }
}
