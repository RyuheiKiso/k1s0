namespace K1s0.Resilience;

/// <summary>
/// Executes asynchronous operations with configurable retry logic, exponential backoff, and jitter.
/// </summary>
public class RetryExecutor
{
    private readonly RetryConfig _config;

    /// <summary>
    /// Initializes a new instance of the <see cref="RetryExecutor"/> class.
    /// </summary>
    /// <param name="config">The retry configuration.</param>
    public RetryExecutor(RetryConfig config)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
    }

    /// <summary>
    /// Executes the specified action with retry logic.
    /// </summary>
    /// <typeparam name="T">The return type of the action.</typeparam>
    /// <param name="action">The asynchronous action to execute.</param>
    /// <returns>The result of the action.</returns>
    /// <exception cref="Exception">The last exception thrown after all retry attempts are exhausted.</exception>
    public async Task<T> ExecuteAsync<T>(Func<Task<T>> action)
    {
        ArgumentNullException.ThrowIfNull(action);

        Exception? lastException = null;

        for (int attempt = 0; attempt < _config.MaxAttempts; attempt++)
        {
            try
            {
                return await action().ConfigureAwait(false);
            }
            catch (Exception ex)
            {
                lastException = ex;

                if (_config.RetryableChecker != null && !_config.RetryableChecker(ex))
                {
                    throw;
                }

                if (attempt + 1 >= _config.MaxAttempts)
                {
                    throw;
                }

                var delayMs = CalculateDelay(attempt);
                await Task.Delay(TimeSpan.FromMilliseconds(delayMs)).ConfigureAwait(false);
            }
        }

        // This line should never be reached, but satisfies the compiler.
        throw lastException!;
    }

    /// <summary>
    /// Calculates the delay in milliseconds for the given attempt using exponential backoff with jitter.
    /// </summary>
    /// <param name="attempt">The zero-based attempt number (0 = first attempt that failed).</param>
    /// <returns>The delay in milliseconds.</returns>
    public double CalculateDelay(int attempt)
    {
        var baseDelay = _config.InitialIntervalMs * Math.Pow(_config.Multiplier, attempt);
        baseDelay = Math.Min(baseDelay, _config.MaxIntervalMs);

        if (_config.JitterFactor > 0)
        {
            var jitter = baseDelay * _config.JitterFactor * (Random.Shared.NextDouble() * 2 - 1);
            baseDelay += jitter;
        }

        return Math.Max(0, baseDelay);
    }
}
