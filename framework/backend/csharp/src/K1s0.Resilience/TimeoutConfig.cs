namespace K1s0.Resilience;

/// <summary>
/// Configuration for timeout behavior.
/// </summary>
/// <param name="DurationSeconds">The timeout duration in seconds. Must be between 0.1 and 300.0. Defaults to 30.0.</param>
public record TimeoutConfig(double DurationSeconds = 30.0)
{
    /// <summary>
    /// Gets the timeout duration in seconds, validated to be within the allowed range.
    /// </summary>
    public double DurationSeconds { get; } = DurationSeconds is >= 0.1 and <= 300.0
        ? DurationSeconds
        : throw new ArgumentOutOfRangeException(nameof(DurationSeconds), DurationSeconds, "DurationSeconds must be between 0.1 and 300.0.");
}
