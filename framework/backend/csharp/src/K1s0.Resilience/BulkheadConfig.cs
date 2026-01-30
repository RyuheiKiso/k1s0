namespace K1s0.Resilience;

/// <summary>
/// Configuration for a named bulkhead partition.
/// </summary>
/// <param name="Name">The name identifying this bulkhead partition.</param>
/// <param name="MaxConcurrent">The maximum number of concurrent executions. Must be at least 1. Defaults to 10.</param>
public record BulkheadConfig(string Name, int MaxConcurrent = 10)
{
    /// <summary>
    /// Gets the name of the bulkhead partition.
    /// </summary>
    public string Name { get; } = !string.IsNullOrWhiteSpace(Name)
        ? Name
        : throw new ArgumentException("Name must not be null or whitespace.", nameof(Name));

    /// <summary>
    /// Gets the maximum number of concurrent executions, validated to be at least 1.
    /// </summary>
    public int MaxConcurrent { get; } = MaxConcurrent >= 1
        ? MaxConcurrent
        : throw new ArgumentOutOfRangeException(nameof(MaxConcurrent), MaxConcurrent, "MaxConcurrent must be at least 1.");
}
