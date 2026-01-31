using System.Globalization;
using Microsoft.AspNetCore.Http;

namespace K1s0.RateLimit;

/// <summary>
/// ASP.NET Core middleware that enforces rate limiting on incoming HTTP requests.
/// Returns HTTP 429 (Too Many Requests) with a Retry-After header when the rate limit is exceeded.
/// </summary>
public class RateLimitMiddleware
{
    private readonly RequestDelegate _next;
    private readonly IRateLimiter _limiter;

    /// <summary>
    /// Initializes a new instance of the <see cref="RateLimitMiddleware"/> class.
    /// </summary>
    /// <param name="next">The next middleware in the pipeline.</param>
    /// <param name="limiter">The rate limiter to enforce.</param>
    public RateLimitMiddleware(RequestDelegate next, IRateLimiter limiter)
    {
        _next = next ?? throw new ArgumentNullException(nameof(next));
        _limiter = limiter ?? throw new ArgumentNullException(nameof(limiter));
    }

    /// <summary>
    /// Processes an HTTP request, rejecting it with 429 if the rate limit is exceeded.
    /// </summary>
    /// <param name="context">The HTTP context for the current request.</param>
    /// <returns>A task representing the asynchronous operation.</returns>
    public async Task InvokeAsync(HttpContext context)
    {
        ArgumentNullException.ThrowIfNull(context);

        if (!await _limiter.TryAcquireAsync(context.RequestAborted).ConfigureAwait(false))
        {
            var retryAfter = _limiter.TimeUntilAvailable();
            context.Response.StatusCode = StatusCodes.Status429TooManyRequests;
            context.Response.Headers["Retry-After"] = Math.Ceiling(retryAfter.TotalSeconds).ToString("F0", CultureInfo.InvariantCulture);
            await context.Response.WriteAsync("Rate limit exceeded", context.RequestAborted).ConfigureAwait(false);
            return;
        }

        await _next(context).ConfigureAwait(false);
    }
}
