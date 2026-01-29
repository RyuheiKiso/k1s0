using System.Diagnostics;
using Grpc.Core;
using Grpc.Core.Interceptors;
using K1s0.Observability;

namespace K1s0.Grpc.Server.Interceptors;

/// <summary>
/// gRPC server interceptor that adds distributed tracing to each call.
/// </summary>
public class TracingInterceptor : Interceptor
{
    /// <inheritdoc />
    public override async Task<TResponse> UnaryServerHandler<TRequest, TResponse>(
        TRequest request,
        ServerCallContext context,
        UnaryServerMethod<TRequest, TResponse> continuation)
    {
        using var activity = K1s0ActivitySource.Instance.StartActivity(
            context.Method,
            ActivityKind.Server);

        activity?.SetTag("rpc.system", "grpc");
        activity?.SetTag("rpc.method", context.Method);

        try
        {
            var response = await continuation(request, context).ConfigureAwait(false);
            activity?.SetTag("rpc.grpc.status_code", (int)StatusCode.OK);
            return response;
        }
        catch (RpcException ex)
        {
            activity?.SetTag("rpc.grpc.status_code", (int)ex.StatusCode);
            activity?.SetStatus(ActivityStatusCode.Error, ex.Message);
            throw;
        }
    }
}
