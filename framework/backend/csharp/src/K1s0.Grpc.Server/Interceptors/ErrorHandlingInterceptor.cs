using System.Net;
using Grpc.Core;
using Grpc.Core.Interceptors;
using K1s0.Error;
using Microsoft.Extensions.Logging;

namespace K1s0.Grpc.Server.Interceptors;

/// <summary>
/// gRPC server interceptor that converts <see cref="K1s0Exception"/> instances
/// to appropriate gRPC status codes.
/// </summary>
public class ErrorHandlingInterceptor : Interceptor
{
    private readonly ILogger<ErrorHandlingInterceptor> _logger;

    /// <summary>
    /// Creates a new <see cref="ErrorHandlingInterceptor"/>.
    /// </summary>
    public ErrorHandlingInterceptor(ILogger<ErrorHandlingInterceptor> logger)
    {
        _logger = logger;
    }

    /// <inheritdoc />
    public override async Task<TResponse> UnaryServerHandler<TRequest, TResponse>(
        TRequest request,
        ServerCallContext context,
        UnaryServerMethod<TRequest, TResponse> continuation)
    {
        try
        {
            return await continuation(request, context).ConfigureAwait(false);
        }
        catch (K1s0Exception ex)
        {
            _logger.LogWarning(ex, "K1s0 error in gRPC call {Method}: {ErrorCode}", context.Method, ex.ErrorCode);
            throw new RpcException(new Status(MapToGrpcStatusCode(ex.HttpStatus), ex.Message));
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Unhandled error in gRPC call {Method}", context.Method);
            throw new RpcException(new Status(StatusCode.Internal, "Internal server error"));
        }
    }

    /// <summary>
    /// Maps an HTTP status code to a gRPC status code.
    /// </summary>
    public static StatusCode MapToGrpcStatusCode(HttpStatusCode httpStatus) => httpStatus switch
    {
        HttpStatusCode.BadRequest => StatusCode.InvalidArgument,
        HttpStatusCode.Unauthorized => StatusCode.Unauthenticated,
        HttpStatusCode.Forbidden => StatusCode.PermissionDenied,
        HttpStatusCode.NotFound => StatusCode.NotFound,
        HttpStatusCode.Conflict => StatusCode.AlreadyExists,
        HttpStatusCode.ServiceUnavailable => StatusCode.Unavailable,
        HttpStatusCode.RequestTimeout or HttpStatusCode.GatewayTimeout => StatusCode.DeadlineExceeded,
        _ => StatusCode.Internal,
    };
}
