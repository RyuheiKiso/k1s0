using System.Diagnostics;
using OpenTelemetry;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;

namespace K1s0.System.Telemetry;

public static class TelemetryInitializer
{
    public static TracerProvider? InitTracing(TelemetryConfig config)
    {
        if (config.Endpoint is null)
        {
            return null;
        }

        return Sdk.CreateTracerProviderBuilder()
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
            })
            .Build();
    }

    public static ActivitySource CreateActivitySource(string serviceName)
    {
        return new ActivitySource(serviceName);
    }
}
