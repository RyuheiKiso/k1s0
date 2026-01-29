using System.Net;

namespace K1s0.Error;

/// <summary>
/// Creates RFC 7807 Problem Details representations from <see cref="K1s0Exception"/> instances.
/// </summary>
public static class ProblemDetailsFactory
{
    /// <summary>
    /// Creates a Problem Details dictionary from a <see cref="K1s0Exception"/>.
    /// </summary>
    /// <param name="exception">The exception to convert.</param>
    /// <returns>A dictionary representing the RFC 7807 Problem Details response body.</returns>
    public static Dictionary<string, object?> Create(K1s0Exception exception)
    {
        ArgumentNullException.ThrowIfNull(exception);

        var statusCode = (int)exception.HttpStatus;
        var details = new Dictionary<string, object?>
        {
            ["status"] = statusCode,
            ["title"] = GetReasonPhrase(exception.HttpStatus),
            ["detail"] = exception.Message,
            ["error_code"] = exception.ErrorCode,
        };

        if (exception.TraceId is not null)
        {
            details["trace_id"] = exception.TraceId;
        }

        if (exception is ValidationException validationException && validationException.Errors.Count > 0)
        {
            details["errors"] = validationException.Errors;
        }

        return details;
    }

    private static string GetReasonPhrase(HttpStatusCode statusCode) => statusCode switch
    {
        HttpStatusCode.BadRequest => "Bad Request",
        HttpStatusCode.Unauthorized => "Unauthorized",
        HttpStatusCode.Forbidden => "Forbidden",
        HttpStatusCode.NotFound => "Not Found",
        HttpStatusCode.Conflict => "Conflict",
        HttpStatusCode.InternalServerError => "Internal Server Error",
        _ => statusCode.ToString(),
    };
}
