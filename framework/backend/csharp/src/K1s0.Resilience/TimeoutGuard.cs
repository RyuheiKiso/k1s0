namespace K1s0.Resilience;

/// <summary>
/// Executes asynchronous operations with a configurable timeout.
/// </summary>
public class TimeoutGuard
{
    private readonly TimeoutConfig _config;

    /// <summary>
    /// Initializes a new instance of the <see cref="TimeoutGuard"/> class.
    /// </summary>
    /// <param name="config">The timeout configuration.</param>
    public TimeoutGuard(TimeoutConfig config)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
    }

    /// <summary>
    /// Executes the specified action with a timeout.
    /// </summary>
    /// <typeparam name="T">The return type of the action.</typeparam>
    /// <param name="action">The asynchronous action to execute. Receives a cancellation token that is cancelled on timeout.</param>
    /// <param name="cancellationToken">An external cancellation token.</param>
    /// <returns>The result of the action.</returns>
    /// <exception cref="K1s0TimeoutException">Thrown when the action exceeds the configured timeout.</exception>
    public async Task<T> ExecuteAsync<T>(Func<CancellationToken, Task<T>> action, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(action);

        using var cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        cts.CancelAfter(TimeSpan.FromSeconds(_config.DurationSeconds));

        try
        {
            return await action(cts.Token).ConfigureAwait(false);
        }
        catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested)
        {
            throw new K1s0TimeoutException($"Operation timed out after {_config.DurationSeconds} seconds.");
        }
    }
}
