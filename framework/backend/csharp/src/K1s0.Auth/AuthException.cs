namespace K1s0.Auth;

/// <summary>
/// Base exception for all authentication and authorization errors in K1s0.
/// </summary>
public class AuthException : Exception
{
    /// <summary>
    /// Gets the structured error code following the k1s0 error code format.
    /// </summary>
    public string ErrorCode { get; }

    /// <summary>
    /// Initializes a new instance of the <see cref="AuthException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="errorCode">The structured error code.</param>
    public AuthException(string message, string errorCode)
        : base(message)
    {
        ErrorCode = errorCode;
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="AuthException"/> class with an inner exception.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="errorCode">The structured error code.</param>
    /// <param name="innerException">The inner exception.</param>
    public AuthException(string message, string errorCode, Exception innerException)
        : base(message, innerException)
    {
        ErrorCode = errorCode;
    }
}

/// <summary>
/// Thrown when a JWT token has expired.
/// </summary>
public class TokenExpiredException : AuthException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="TokenExpiredException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public TokenExpiredException(string message = "Token has expired")
        : base(message, "auth.token_expired")
    {
    }
}

/// <summary>
/// Thrown when a JWT token is invalid (malformed, bad signature, etc.).
/// </summary>
public class TokenInvalidException : AuthException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="TokenInvalidException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public TokenInvalidException(string message = "Token is invalid")
        : base(message, "auth.token_invalid")
    {
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="TokenInvalidException"/> class with an inner exception.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The inner exception.</param>
    public TokenInvalidException(string message, Exception innerException)
        : base(message, "auth.token_invalid", innerException)
    {
    }
}

/// <summary>
/// Thrown when the authenticated user lacks the required permissions for an operation.
/// </summary>
public class InsufficientPermissionException : AuthException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="InsufficientPermissionException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public InsufficientPermissionException(string message = "Insufficient permissions")
        : base(message, "auth.insufficient_permission")
    {
    }
}

/// <summary>
/// Thrown when OIDC discovery fails (network error, invalid response, etc.).
/// </summary>
public class DiscoveryException : AuthException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="DiscoveryException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public DiscoveryException(string message = "OIDC discovery failed")
        : base(message, "auth.discovery_failed")
    {
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="DiscoveryException"/> class with an inner exception.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The inner exception.</param>
    public DiscoveryException(string message, Exception innerException)
        : base(message, "auth.discovery_failed", innerException)
    {
    }
}
