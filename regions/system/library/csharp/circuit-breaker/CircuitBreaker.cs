namespace K1s0.System.CircuitBreaker;

public class CircuitBreaker
{
    private readonly CircuitBreakerConfig _config;
    private readonly object _lock = new();
    private CircuitState _state = CircuitState.Closed;
    private int _failureCount;
    private int _successCount;
    private DateTime _openedAt;

    public CircuitBreaker(CircuitBreakerConfig config)
    {
        _config = config;
    }

    public CircuitState State
    {
        get
        {
            lock (_lock)
            {
                if (_state == CircuitState.Open && DateTime.UtcNow - _openedAt >= _config.Timeout)
                {
                    _state = CircuitState.HalfOpen;
                }

                return _state;
            }
        }
    }

    public bool IsOpen => State == CircuitState.Open;

    public void RecordSuccess()
    {
        lock (_lock)
        {
            if (_state == CircuitState.HalfOpen)
            {
                _successCount++;
                if (_successCount >= _config.SuccessThreshold)
                {
                    _state = CircuitState.Closed;
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

    public void RecordFailure()
    {
        lock (_lock)
        {
            if (_state == CircuitState.HalfOpen)
            {
                _state = CircuitState.Open;
                _openedAt = DateTime.UtcNow;
                _successCount = 0;
            }
            else if (_state == CircuitState.Closed)
            {
                _failureCount++;
                if (_failureCount >= _config.FailureThreshold)
                {
                    _state = CircuitState.Open;
                    _openedAt = DateTime.UtcNow;
                }
            }
        }
    }

    public async Task<T> CallAsync<T>(Func<Task<T>> fn, CancellationToken ct = default)
    {
        if (State == CircuitState.Open)
        {
            throw new CircuitBreakerOpenException();
        }

        try
        {
            var result = await fn().ConfigureAwait(false);
            RecordSuccess();
            return result;
        }
        catch (Exception)
        {
            RecordFailure();
            throw;
        }
    }
}
