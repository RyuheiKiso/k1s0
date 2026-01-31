namespace K1s0.Consensus;

/// <summary>
/// Backoff strategy for retry policies.
/// </summary>
public enum BackoffStrategy
{
    /// <summary>
    /// Fixed delay between retries.
    /// </summary>
    Fixed,

    /// <summary>
    /// Linearly increasing delay between retries.
    /// </summary>
    Linear,

    /// <summary>
    /// Exponentially increasing delay between retries.
    /// </summary>
    Exponential
}

/// <summary>
/// Defines the retry policy for saga steps.
/// </summary>
public sealed class RetryPolicy
{
    /// <summary>
    /// Maximum number of retry attempts. Default is 3.
    /// </summary>
    public int MaxRetries { get; init; } = 3;

    /// <summary>
    /// Base delay between retries. Default is 1 second.
    /// </summary>
    public TimeSpan BaseDelay { get; init; } = TimeSpan.FromSeconds(1);

    /// <summary>
    /// Maximum delay between retries. Default is 30 seconds.
    /// </summary>
    public TimeSpan MaxDelay { get; init; } = TimeSpan.FromSeconds(30);

    /// <summary>
    /// The backoff strategy. Default is <see cref="BackoffStrategy.Exponential"/>.
    /// </summary>
    public BackoffStrategy Strategy { get; init; } = BackoffStrategy.Exponential;

    /// <summary>
    /// A retry policy that performs no retries.
    /// </summary>
    public static RetryPolicy None => new() { MaxRetries = 0 };

    /// <summary>
    /// Calculates the delay for the given attempt number (0-based).
    /// </summary>
    /// <param name="attempt">The zero-based attempt number.</param>
    /// <returns>The delay before the next retry.</returns>
    public TimeSpan GetDelay(int attempt)
    {
        var delay = Strategy switch
        {
            BackoffStrategy.Fixed => BaseDelay,
            BackoffStrategy.Linear => TimeSpan.FromMilliseconds(BaseDelay.TotalMilliseconds * (attempt + 1)),
            BackoffStrategy.Exponential => TimeSpan.FromMilliseconds(BaseDelay.TotalMilliseconds * Math.Pow(2, attempt)),
            _ => BaseDelay
        };

        return delay > MaxDelay ? MaxDelay : delay;
    }
}
