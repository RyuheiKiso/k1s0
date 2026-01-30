namespace K1s0.Resilience;

/// <summary>
/// Configuration for concurrency limiting.
/// </summary>
/// <param name="MaxConcurrent">The maximum number of concurrent executions. Must be at least 1. Defaults to 10.</param>
public record ConcurrencyConfig(int MaxConcurrent = 10)
{
    /// <summary>
    /// Gets the maximum number of concurrent executions, validated to be at least 1.
    /// </summary>
    public int MaxConcurrent { get; } = MaxConcurrent >= 1
        ? MaxConcurrent
        : throw new ArgumentOutOfRangeException(nameof(MaxConcurrent), MaxConcurrent, "MaxConcurrent must be at least 1.");
}
