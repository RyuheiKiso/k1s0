namespace K1s0.Resilience;

/// <summary>
/// Base exception for all resilience-related failures.
/// </summary>
public class ResilienceException : Exception
{
    /// <summary>
    /// Gets the error code identifying the type of resilience failure.
    /// </summary>
    public string ErrorCode { get; }

    /// <summary>
    /// Gets a value indicating whether the operation that caused this exception can be retried.
    /// </summary>
    public bool IsRetryable { get; }

    /// <summary>
    /// Initializes a new instance of the <see cref="ResilienceException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="errorCode">The error code identifying the failure type.</param>
    /// <param name="isRetryable">Whether the operation can be retried.</param>
    public ResilienceException(string message, string errorCode, bool isRetryable)
        : base(message)
    {
        ErrorCode = errorCode;
        IsRetryable = isRetryable;
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="ResilienceException"/> class with an inner exception.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="errorCode">The error code identifying the failure type.</param>
    /// <param name="isRetryable">Whether the operation can be retried.</param>
    /// <param name="innerException">The inner exception.</param>
    public ResilienceException(string message, string errorCode, bool isRetryable, Exception innerException)
        : base(message, innerException)
    {
        ErrorCode = errorCode;
        IsRetryable = isRetryable;
    }
}

/// <summary>
/// Exception thrown when an operation exceeds its configured timeout duration.
/// This exception is retryable by default.
/// </summary>
public class K1s0TimeoutException : ResilienceException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="K1s0TimeoutException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public K1s0TimeoutException(string message)
        : base(message, "resilience.timeout", isRetryable: true)
    {
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="K1s0TimeoutException"/> class with an inner exception.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The inner exception.</param>
    public K1s0TimeoutException(string message, Exception innerException)
        : base(message, "resilience.timeout", isRetryable: true, innerException)
    {
    }
}

/// <summary>
/// Exception thrown when a circuit breaker is in the open state and rejects execution.
/// This exception is not retryable.
/// </summary>
public class CircuitOpenException : ResilienceException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="CircuitOpenException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public CircuitOpenException(string message)
        : base(message, "resilience.circuit_open", isRetryable: false)
    {
    }
}

/// <summary>
/// Exception thrown when a concurrency limiter rejects execution due to reaching its limit.
/// This exception is not retryable.
/// </summary>
public class ConcurrencyLimitException : ResilienceException
{
    /// <summary>
    /// Initializes a new instance of the <see cref="ConcurrencyLimitException"/> class.
    /// </summary>
    /// <param name="message">The error message.</param>
    public ConcurrencyLimitException(string message)
        : base(message, "resilience.concurrency_limit", isRetryable: false)
    {
    }
}
