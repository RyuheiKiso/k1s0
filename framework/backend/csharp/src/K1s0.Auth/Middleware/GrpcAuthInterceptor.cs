using Grpc.Core;
using Grpc.Core.Interceptors;
using K1s0.Auth.Jwt;
using Microsoft.Extensions.Logging;

namespace K1s0.Auth.Middleware;

/// <summary>
/// gRPC server interceptor that validates JWT Bearer tokens from call metadata.
/// </summary>
public class GrpcAuthInterceptor : Interceptor
{
    private readonly JwtVerifier _verifier;
    private readonly ILogger<GrpcAuthInterceptor> _logger;

    /// <summary>
    /// Initializes a new instance of the <see cref="GrpcAuthInterceptor"/> class.
    /// </summary>
    /// <param name="verifier">The JWT verifier.</param>
    /// <param name="logger">The logger.</param>
    public GrpcAuthInterceptor(JwtVerifier verifier, ILogger<GrpcAuthInterceptor> logger)
    {
        _verifier = verifier;
        _logger = logger;
    }

    /// <inheritdoc />
    public override async Task<TResponse> UnaryServerHandler<TRequest, TResponse>(
        TRequest request,
        ServerCallContext context,
        UnaryServerMethod<TRequest, TResponse> continuation)
    {
        var claims = await AuthenticateAsync(context).ConfigureAwait(false);
        context.UserState["Claims"] = claims;
        return await continuation(request, context).ConfigureAwait(false);
    }

    private async Task<Claims> AuthenticateAsync(ServerCallContext context)
    {
        var authValue = context.RequestHeaders.GetValue("authorization");
        if (string.IsNullOrEmpty(authValue) || !authValue.StartsWith("Bearer ", StringComparison.OrdinalIgnoreCase))
        {
            _logger.LogWarning("Missing or invalid authorization metadata");
            throw new RpcException(new Status(StatusCode.Unauthenticated, "Missing or invalid authorization token"));
        }

        var token = authValue["Bearer ".Length..].Trim();

        try
        {
            return await _verifier.VerifyAsync(token, context.CancellationToken).ConfigureAwait(false);
        }
        catch (TokenExpiredException)
        {
            _logger.LogWarning("Token expired");
            throw new RpcException(new Status(StatusCode.Unauthenticated, "Token expired"));
        }
        catch (TokenInvalidException ex)
        {
            _logger.LogWarning("Token invalid: {Message}", ex.Message);
            throw new RpcException(new Status(StatusCode.Unauthenticated, "Invalid token"));
        }
    }
}
