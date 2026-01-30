using K1s0.Auth.Jwt;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Logging;

namespace K1s0.Auth.Middleware;

/// <summary>
/// ASP.NET Core middleware that validates JWT Bearer tokens on incoming requests.
/// Skips health check paths (/healthz, /readyz).
/// </summary>
public class AuthMiddleware
{
    private readonly RequestDelegate _next;
    private readonly JwtVerifier _verifier;
    private readonly ILogger<AuthMiddleware> _logger;

    private static readonly HashSet<string> SkipPaths = new(StringComparer.OrdinalIgnoreCase)
    {
        "/healthz",
        "/readyz",
    };

    /// <summary>
    /// Initializes a new instance of the <see cref="AuthMiddleware"/> class.
    /// </summary>
    /// <param name="next">The next middleware in the pipeline.</param>
    /// <param name="verifier">The JWT verifier.</param>
    /// <param name="logger">The logger.</param>
    public AuthMiddleware(RequestDelegate next, JwtVerifier verifier, ILogger<AuthMiddleware> logger)
    {
        _next = next;
        _verifier = verifier;
        _logger = logger;
    }

    /// <summary>
    /// Processes the HTTP request, validating the JWT Bearer token.
    /// </summary>
    /// <param name="context">The HTTP context.</param>
    /// <returns>A task representing the asynchronous operation.</returns>
    public async Task InvokeAsync(HttpContext context)
    {
        if (SkipPaths.Contains(context.Request.Path.Value ?? string.Empty))
        {
            await _next(context).ConfigureAwait(false);
            return;
        }

        var authHeader = context.Request.Headers.Authorization.ToString();
        if (string.IsNullOrEmpty(authHeader) || !authHeader.StartsWith("Bearer ", StringComparison.OrdinalIgnoreCase))
        {
            _logger.LogWarning("Missing or invalid Authorization header");
            context.Response.StatusCode = StatusCodes.Status401Unauthorized;
            return;
        }

        var token = authHeader["Bearer ".Length..].Trim();

        try
        {
            var claims = await _verifier.VerifyAsync(token, context.RequestAborted).ConfigureAwait(false);
            context.Items["Claims"] = claims;
            await _next(context).ConfigureAwait(false);
        }
        catch (TokenExpiredException)
        {
            _logger.LogWarning("Token expired");
            context.Response.StatusCode = StatusCodes.Status401Unauthorized;
        }
        catch (TokenInvalidException ex)
        {
            _logger.LogWarning("Token invalid: {Message}", ex.Message);
            context.Response.StatusCode = StatusCodes.Status401Unauthorized;
        }
    }
}
