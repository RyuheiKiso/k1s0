namespace K1s0.Resilience;

/// <summary>
/// Implements the circuit breaker pattern to prevent cascading failures.
/// Thread-safe state machine that transitions between Closed, Open, and HalfOpen states.
/// </summary>
public class CircuitBreaker
{
    private readonly CircuitBreakerConfig _config;
    private readonly object _lock = new();
    private CircuitState _state = CircuitState.Closed;
    private int _failureCount;
    private int _successCount;
    private DateTime _openedAt;
    private long _rejectedCount;
    private long _stateTransitionCount;

    /// <summary>
    /// Gets the current state of the circuit breaker.
    /// </summary>
    public CircuitState State
    {
        get
        {
            lock (_lock)
            {
                return _state;
            }
        }
    }

    /// <summary>
    /// Gets the total number of rejected executions due to the circuit being open.
    /// </summary>
    public long RejectedCount
    {
        get
        {
            lock (_lock)
            {
                return _rejectedCount;
            }
        }
    }

    /// <summary>
    /// Gets the total number of state transitions that have occurred.
    /// </summary>
    public long StateTransitionCount
    {
        get
        {
            lock (_lock)
            {
                return _stateTransitionCount;
            }
        }
    }

    /// <summary>
    /// Initializes a new instance of the <see cref="CircuitBreaker"/> class.
    /// </summary>
    /// <param name="config">The circuit breaker configuration.</param>
    public CircuitBreaker(CircuitBreakerConfig config)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
    }

    /// <summary>
    /// Executes the specified action if the circuit allows it.
    /// </summary>
    /// <typeparam name="T">The return type of the action.</typeparam>
    /// <param name="action">The asynchronous action to execute.</param>
    /// <returns>The result of the action.</returns>
    /// <exception cref="CircuitOpenException">Thrown when the circuit is open and the reset timeout has not elapsed.</exception>
    public async Task<T> ExecuteAsync<T>(Func<Task<T>> action)
    {
        ArgumentNullException.ThrowIfNull(action);

        lock (_lock)
        {
            if (_state == CircuitState.Open)
            {
                if (DateTime.UtcNow - _openedAt >= TimeSpan.FromSeconds(_config.ResetTimeoutSeconds))
                {
                    TransitionTo(CircuitState.HalfOpen);
                    _successCount = 0;
                }
                else
                {
                    _rejectedCount++;
                    throw new CircuitOpenException("Circuit breaker is open. Requests are being rejected.");
                }
            }
        }

        try
        {
            var result = await action().ConfigureAwait(false);
            OnSuccess();
            return result;
        }
        catch (Exception ex)
        {
            OnFailure(ex);
            throw;
        }
    }

    private void OnSuccess()
    {
        lock (_lock)
        {
            if (_state == CircuitState.HalfOpen)
            {
                _successCount++;
                if (_successCount >= _config.SuccessThreshold)
                {
                    TransitionTo(CircuitState.Closed);
                    _failureCount = 0;
                    _successCount = 0;
                }
            }
            else if (_state == CircuitState.Closed)
            {
                _failureCount = 0;
            }
        }
    }

    private void OnFailure(Exception ex)
    {
        var predicate = _config.FailurePredicate;
        if (predicate != null && !predicate(ex))
        {
            return;
        }

        lock (_lock)
        {
            if (_state == CircuitState.HalfOpen)
            {
                TransitionTo(CircuitState.Open);
                _openedAt = DateTime.UtcNow;
                _successCount = 0;
            }
            else if (_state == CircuitState.Closed)
            {
                _failureCount++;
                if (_failureCount >= _config.FailureThreshold)
                {
                    TransitionTo(CircuitState.Open);
                    _openedAt = DateTime.UtcNow;
                }
            }
        }
    }

    private void TransitionTo(CircuitState newState)
    {
        _state = newState;
        _stateTransitionCount++;
    }
}
