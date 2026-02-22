using System.Diagnostics;
using Grpc.Core;
using Grpc.Core.Interceptors;

namespace K1s0.System.Telemetry.Middleware;

public sealed class GrpcTelemetryInterceptor : Interceptor
{
    private readonly K1s0Metrics _metrics;

    public GrpcTelemetryInterceptor(K1s0Metrics metrics)
    {
        _metrics = metrics;
    }

    public override async Task<TResponse> UnaryServerHandler<TRequest, TResponse>(
        TRequest request,
        ServerCallContext context,
        UnaryServerMethod<TRequest, TResponse> continuation)
    {
        _metrics.IncrementInFlight();
        var stopwatch = Stopwatch.StartNew();
        int statusCode = 0;

        try
        {
            var response = await continuation(request, context);
            return response;
        }
        catch (RpcException ex)
        {
            statusCode = (int)ex.StatusCode;
            throw;
        }
        finally
        {
            stopwatch.Stop();
            _metrics.DecrementInFlight();
            _metrics.RecordRequest(
                "gRPC",
                context.Method,
                statusCode,
                stopwatch.Elapsed.TotalMilliseconds);
        }
    }
}
