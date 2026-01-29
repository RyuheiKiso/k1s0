using System.Diagnostics;
using System.Net;

namespace K1s0.Error;

/// <summary>
/// Base exception for all k1s0 application errors.
/// Carries a structured error code, HTTP status, and trace identifier.
/// </summary>
public class K1s0Exception : Exception
{
    /// <summary>
    /// The structured error code (e.g., "user.auth.invalid_credentials").
    /// </summary>
    public string ErrorCode { get; }

    /// <summary>
    /// The HTTP status code that should be returned for this error.
    /// </summary>
    public HttpStatusCode HttpStatus { get; }

    /// <summary>
    /// The distributed trace identifier, captured automatically from <see cref="Activity.Current"/>.
    /// </summary>
    public string? TraceId { get; }

    /// <summary>
    /// Creates a new <see cref="K1s0Exception"/>.
    /// </summary>
    /// <param name="errorCode">Structured error code in "{service}.{category}.{reason}" format.</param>
    /// <param name="message">Human-readable error message.</param>
    /// <param name="httpStatus">HTTP status code for this error.</param>
    /// <param name="innerException">Optional inner exception.</param>
    public K1s0Exception(
        string errorCode,
        string message,
        HttpStatusCode httpStatus = HttpStatusCode.InternalServerError,
        Exception? innerException = null)
        : base(message, innerException)
    {
        ErrorCode = errorCode;
        HttpStatus = httpStatus;
        TraceId = Activity.Current?.TraceId.ToString();
    }
}

/// <summary>
/// Represents a "not found" error (HTTP 404).
/// </summary>
public class NotFoundException : K1s0Exception
{
    /// <summary>
    /// Creates a new <see cref="NotFoundException"/>.
    /// </summary>
    public NotFoundException(string errorCode, string message, Exception? innerException = null)
        : base(errorCode, message, HttpStatusCode.NotFound, innerException)
    {
    }
}

/// <summary>
/// Represents a validation error (HTTP 400).
/// </summary>
public class ValidationException : K1s0Exception
{
    /// <summary>
    /// Validation errors keyed by field name.
    /// </summary>
    public IReadOnlyDictionary<string, string[]> Errors { get; }

    /// <summary>
    /// Creates a new <see cref="ValidationException"/>.
    /// </summary>
    public ValidationException(
        string errorCode,
        string message,
        IReadOnlyDictionary<string, string[]>? errors = null,
        Exception? innerException = null)
        : base(errorCode, message, HttpStatusCode.BadRequest, innerException)
    {
        Errors = errors ?? new Dictionary<string, string[]>();
    }
}

/// <summary>
/// Represents a conflict error (HTTP 409).
/// </summary>
public class ConflictException : K1s0Exception
{
    /// <summary>
    /// Creates a new <see cref="ConflictException"/>.
    /// </summary>
    public ConflictException(string errorCode, string message, Exception? innerException = null)
        : base(errorCode, message, HttpStatusCode.Conflict, innerException)
    {
    }
}

/// <summary>
/// Represents an authentication error (HTTP 401).
/// </summary>
public class UnauthorizedException : K1s0Exception
{
    /// <summary>
    /// Creates a new <see cref="UnauthorizedException"/>.
    /// </summary>
    public UnauthorizedException(string errorCode, string message, Exception? innerException = null)
        : base(errorCode, message, HttpStatusCode.Unauthorized, innerException)
    {
    }
}

/// <summary>
/// Represents an authorization error (HTTP 403).
/// </summary>
public class ForbiddenException : K1s0Exception
{
    /// <summary>
    /// Creates a new <see cref="ForbiddenException"/>.
    /// </summary>
    public ForbiddenException(string errorCode, string message, Exception? innerException = null)
        : base(errorCode, message, HttpStatusCode.Forbidden, innerException)
    {
    }
}
