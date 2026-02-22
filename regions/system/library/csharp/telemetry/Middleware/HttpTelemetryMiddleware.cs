using System.Diagnostics;
using Microsoft.AspNetCore.Http;

namespace K1s0.System.Telemetry.Middleware;

public sealed class HttpTelemetryMiddleware : IMiddleware
{
    private readonly K1s0Metrics _metrics;

    public HttpTelemetryMiddleware(K1s0Metrics metrics)
    {
        _metrics = metrics;
    }

    public async Task InvokeAsync(HttpContext context, RequestDelegate next)
    {
        _metrics.IncrementInFlight();
        var stopwatch = Stopwatch.StartNew();

        try
        {
            await next(context);
        }
        finally
        {
            stopwatch.Stop();
            _metrics.DecrementInFlight();
            _metrics.RecordRequest(
                context.Request.Method,
                context.Request.Path.Value ?? "/",
                context.Response.StatusCode,
                stopwatch.Elapsed.TotalMilliseconds);
        }
    }
}
