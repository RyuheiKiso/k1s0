namespace K1s0.Resilience;

/// <summary>
/// Configuration for circuit breaker behavior.
/// </summary>
/// <param name="FailureThreshold">Number of consecutive failures before opening the circuit. Defaults to 5.</param>
/// <param name="SuccessThreshold">Number of consecutive successes in half-open state before closing the circuit. Defaults to 3.</param>
/// <param name="ResetTimeoutSeconds">Duration in seconds the circuit remains open before transitioning to half-open. Defaults to 60.0.</param>
/// <param name="FailurePredicate">Optional predicate to determine whether an exception counts as a failure. Defaults to all exceptions.</param>
public record CircuitBreakerConfig(
    int FailureThreshold = 5,
    int SuccessThreshold = 3,
    double ResetTimeoutSeconds = 60.0,
    Func<Exception, bool>? FailurePredicate = null)
{
    /// <summary>
    /// Gets the failure threshold, validated to be at least 1.
    /// </summary>
    public int FailureThreshold { get; } = FailureThreshold >= 1
        ? FailureThreshold
        : throw new ArgumentOutOfRangeException(nameof(FailureThreshold), FailureThreshold, "FailureThreshold must be at least 1.");

    /// <summary>
    /// Gets the success threshold, validated to be at least 1.
    /// </summary>
    public int SuccessThreshold { get; } = SuccessThreshold >= 1
        ? SuccessThreshold
        : throw new ArgumentOutOfRangeException(nameof(SuccessThreshold), SuccessThreshold, "SuccessThreshold must be at least 1.");

    /// <summary>
    /// Gets the reset timeout in seconds, validated to be positive.
    /// </summary>
    public double ResetTimeoutSeconds { get; } = ResetTimeoutSeconds > 0
        ? ResetTimeoutSeconds
        : throw new ArgumentOutOfRangeException(nameof(ResetTimeoutSeconds), ResetTimeoutSeconds, "ResetTimeoutSeconds must be positive.");
}
