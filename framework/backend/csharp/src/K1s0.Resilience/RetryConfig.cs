namespace K1s0.Resilience;

/// <summary>
/// Configuration for retry behavior with exponential backoff and jitter.
/// </summary>
/// <param name="MaxAttempts">Maximum number of attempts including the initial attempt. Must be at least 1. Defaults to 3.</param>
/// <param name="InitialIntervalMs">Initial delay between retries in milliseconds. Must be positive. Defaults to 1000.</param>
/// <param name="MaxIntervalMs">Maximum delay between retries in milliseconds. Must be positive. Defaults to 60000.</param>
/// <param name="Multiplier">Multiplier applied to the delay for each subsequent retry. Must be at least 1.0. Defaults to 2.0.</param>
/// <param name="JitterFactor">Random jitter factor applied to delays (0.0 to 1.0). Defaults to 0.1.</param>
/// <param name="RetryableChecker">Optional predicate to determine whether an exception is retryable. Defaults to all exceptions.</param>
public record RetryConfig(
    int MaxAttempts = 3,
    double InitialIntervalMs = 1000,
    double MaxIntervalMs = 60000,
    double Multiplier = 2.0,
    double JitterFactor = 0.1,
    Func<Exception, bool>? RetryableChecker = null)
{
    /// <summary>
    /// Gets the maximum number of attempts, validated to be at least 1.
    /// </summary>
    public int MaxAttempts { get; } = MaxAttempts >= 1
        ? MaxAttempts
        : throw new ArgumentOutOfRangeException(nameof(MaxAttempts), MaxAttempts, "MaxAttempts must be at least 1.");

    /// <summary>
    /// Gets the initial interval in milliseconds, validated to be positive.
    /// </summary>
    public double InitialIntervalMs { get; } = InitialIntervalMs > 0
        ? InitialIntervalMs
        : throw new ArgumentOutOfRangeException(nameof(InitialIntervalMs), InitialIntervalMs, "InitialIntervalMs must be positive.");

    /// <summary>
    /// Gets the maximum interval in milliseconds, validated to be positive.
    /// </summary>
    public double MaxIntervalMs { get; } = MaxIntervalMs > 0
        ? MaxIntervalMs
        : throw new ArgumentOutOfRangeException(nameof(MaxIntervalMs), MaxIntervalMs, "MaxIntervalMs must be positive.");

    /// <summary>
    /// Gets the multiplier, validated to be at least 1.0.
    /// </summary>
    public double Multiplier { get; } = Multiplier >= 1.0
        ? Multiplier
        : throw new ArgumentOutOfRangeException(nameof(Multiplier), Multiplier, "Multiplier must be at least 1.0.");

    /// <summary>
    /// Gets the jitter factor, validated to be between 0.0 and 1.0.
    /// </summary>
    public double JitterFactor { get; } = JitterFactor is >= 0.0 and <= 1.0
        ? JitterFactor
        : throw new ArgumentOutOfRangeException(nameof(JitterFactor), JitterFactor, "JitterFactor must be between 0.0 and 1.0.");
}
