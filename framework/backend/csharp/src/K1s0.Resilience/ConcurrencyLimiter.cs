namespace K1s0.Resilience;

/// <summary>
/// Limits the number of concurrent executions using a semaphore.
/// </summary>
public class ConcurrencyLimiter : IDisposable
{
    private readonly SemaphoreSlim _semaphore;
    private bool _disposed;

    /// <summary>
    /// Gets the concurrency metrics for this limiter.
    /// </summary>
    public ConcurrencyMetrics Metrics { get; } = new();

    /// <summary>
    /// Initializes a new instance of the <see cref="ConcurrencyLimiter"/> class.
    /// </summary>
    /// <param name="config">The concurrency configuration.</param>
    public ConcurrencyLimiter(ConcurrencyConfig config)
    {
        ArgumentNullException.ThrowIfNull(config);
        _semaphore = new SemaphoreSlim(config.MaxConcurrent, config.MaxConcurrent);
    }

    /// <summary>
    /// Executes the specified action if the concurrency limit has not been reached.
    /// </summary>
    /// <typeparam name="T">The return type of the action.</typeparam>
    /// <param name="action">The asynchronous action to execute.</param>
    /// <returns>The result of the action.</returns>
    /// <exception cref="ConcurrencyLimitException">Thrown when the concurrency limit has been reached.</exception>
    public async Task<T> ExecuteAsync<T>(Func<Task<T>> action)
    {
        ArgumentNullException.ThrowIfNull(action);
        ObjectDisposedException.ThrowIf(_disposed, this);

        if (!_semaphore.Wait(0))
        {
            Metrics.IncrementRejected();
            throw new ConcurrencyLimitException("Concurrency limit reached. Unable to execute action.");
        }

        Metrics.IncrementActive();
        try
        {
            return await action().ConfigureAwait(false);
        }
        finally
        {
            Metrics.DecrementActive();
            _semaphore.Release();
        }
    }

    /// <summary>
    /// Releases the resources used by the <see cref="ConcurrencyLimiter"/>.
    /// </summary>
    public void Dispose()
    {
        if (!_disposed)
        {
            _disposed = true;
            _semaphore.Dispose();
        }

        GC.SuppressFinalize(this);
    }
}
