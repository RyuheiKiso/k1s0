namespace K1s0.System.Retry;

public enum CircuitBreakerState
{
    Closed,
    Open,
    HalfOpen,
}

public class CircuitBreakerConfig
{
    public int FailureThreshold { get; init; } = 5;

    public int SuccessThreshold { get; init; } = 2;

    public TimeSpan Timeout { get; init; } = TimeSpan.FromSeconds(30);
}

public class CircuitBreaker
{
    private readonly CircuitBreakerConfig _config;
    private readonly SemaphoreSlim _lock = new(1, 1);
    private CircuitBreakerState _state = CircuitBreakerState.Closed;
    private int _failureCount;
    private int _successCount;
    private DateTime _openedAt;

    public CircuitBreaker(CircuitBreakerConfig config)
    {
        _config = config;
    }

    public CircuitBreakerState State => _state;

    public bool IsOpen()
    {
        if (_state == CircuitBreakerState.Open)
        {
            if (DateTime.UtcNow - _openedAt >= _config.Timeout)
            {
                _state = CircuitBreakerState.HalfOpen;
                _successCount = 0;
                return false;
            }

            return true;
        }

        return false;
    }

    public async Task RecordSuccessAsync()
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_state == CircuitBreakerState.HalfOpen)
            {
                _successCount++;
                if (_successCount >= _config.SuccessThreshold)
                {
                    _state = CircuitBreakerState.Closed;
                    _failureCount = 0;
                    _successCount = 0;
                }
            }
            else
            {
                _failureCount = 0;
            }
        }
        finally
        {
            _lock.Release();
        }
    }

    public async Task RecordFailureAsync()
    {
        await _lock.WaitAsync().ConfigureAwait(false);
        try
        {
            if (_state == CircuitBreakerState.HalfOpen)
            {
                _state = CircuitBreakerState.Open;
                _openedAt = DateTime.UtcNow;
                _successCount = 0;
                return;
            }

            _failureCount++;
            if (_failureCount >= _config.FailureThreshold)
            {
                _state = CircuitBreakerState.Open;
                _openedAt = DateTime.UtcNow;
            }
        }
        finally
        {
            _lock.Release();
        }
    }
}
