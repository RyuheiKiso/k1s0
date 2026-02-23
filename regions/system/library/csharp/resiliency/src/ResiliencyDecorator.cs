namespace K1s0.System.Resiliency;

public sealed class ResiliencyDecorator : IDisposable
{
    private readonly ResiliencyPolicy _policy;
    private readonly Bulkhead? _bulkhead;
    private readonly object _cbLock = new();

    private enum CircuitState
    {
        Closed,
        Open,
        HalfOpen,
    }

    private CircuitState _cbState = CircuitState.Closed;
    private int _cbFailureCount;
    private int _cbSuccessCount;
    private DateTime _cbLastFailureTime;

    public ResiliencyDecorator(ResiliencyPolicy policy)
    {
        _policy = policy;
        if (policy.Bulkhead is not null)
        {
            _bulkhead = new Bulkhead(
                policy.Bulkhead.MaxConcurrentCalls,
                policy.Bulkhead.MaxWaitDuration);
        }
    }

    public async Task<T> ExecuteAsync<T>(
        Func<CancellationToken, Task<T>> fn,
        CancellationToken ct = default)
    {
        CheckCircuitBreaker();

        if (_bulkhead is not null)
        {
            await _bulkhead.AcquireAsync(ct);
        }

        try
        {
            return await ExecuteWithRetryAsync(fn, ct);
        }
        finally
        {
            _bulkhead?.Release();
        }
    }

    public void Dispose() => _bulkhead?.Dispose();

    private async Task<T> ExecuteWithRetryAsync<T>(
        Func<CancellationToken, Task<T>> fn,
        CancellationToken ct)
    {
        var maxAttempts = _policy.Retry?.MaxAttempts ?? 1;
        Exception? lastException = null;

        for (var attempt = 0; attempt < maxAttempts; attempt++)
        {
            try
            {
                var result = await ExecuteWithTimeoutAsync(fn, ct);
                RecordSuccess();
                return result;
            }
            catch (ResiliencyException)
            {
                throw;
            }
            catch (Exception ex)
            {
                RecordFailure();
                lastException = ex;

                CheckCircuitBreaker();

                if (attempt + 1 < maxAttempts && _policy.Retry is not null)
                {
                    var delay = CalculateBackoff(
                        attempt,
                        _policy.Retry.BaseDelay,
                        _policy.Retry.MaxDelay);
                    await Task.Delay(delay, ct);
                }
            }
        }

        throw new ResiliencyException(
            $"Max retries exceeded after {maxAttempts} attempts",
            ResiliencyErrorKind.MaxRetriesExceeded,
            lastException!);
    }

    private async Task<T> ExecuteWithTimeoutAsync<T>(
        Func<CancellationToken, Task<T>> fn,
        CancellationToken ct)
    {
        if (_policy.Timeout is null)
        {
            return await fn(ct);
        }

        using var cts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        cts.CancelAfter(_policy.Timeout.Value);

        try
        {
            return await fn(cts.Token);
        }
        catch (OperationCanceledException) when (!ct.IsCancellationRequested)
        {
            throw new ResiliencyException(
                $"Timed out after {_policy.Timeout.Value.TotalMilliseconds}ms",
                ResiliencyErrorKind.Timeout);
        }
    }

    private void CheckCircuitBreaker()
    {
        if (_policy.CircuitBreaker is null)
        {
            return;
        }

        lock (_cbLock)
        {
            switch (_cbState)
            {
                case CircuitState.Closed:
                    return;

                case CircuitState.Open:
                    var elapsed = DateTime.UtcNow - _cbLastFailureTime;
                    if (elapsed >= _policy.CircuitBreaker.RecoveryTimeout)
                    {
                        _cbState = CircuitState.HalfOpen;
                        _cbSuccessCount = 0;
                        return;
                    }

                    var remaining = _policy.CircuitBreaker.RecoveryTimeout - elapsed;
                    throw new ResiliencyException(
                        $"Circuit breaker open, remaining: {remaining.TotalMilliseconds}ms",
                        ResiliencyErrorKind.CircuitBreakerOpen);

                case CircuitState.HalfOpen:
                    return;
            }
        }
    }

    private void RecordSuccess()
    {
        if (_policy.CircuitBreaker is null)
        {
            return;
        }

        lock (_cbLock)
        {
            switch (_cbState)
            {
                case CircuitState.HalfOpen:
                    _cbSuccessCount++;
                    if (_cbSuccessCount >= _policy.CircuitBreaker.HalfOpenMaxCalls)
                    {
                        _cbState = CircuitState.Closed;
                        _cbFailureCount = 0;
                    }

                    break;

                case CircuitState.Closed:
                    _cbFailureCount = 0;
                    break;
            }
        }
    }

    private void RecordFailure()
    {
        if (_policy.CircuitBreaker is null)
        {
            return;
        }

        lock (_cbLock)
        {
            _cbFailureCount++;
            if (_cbFailureCount >= _policy.CircuitBreaker.FailureThreshold)
            {
                _cbState = CircuitState.Open;
                _cbLastFailureTime = DateTime.UtcNow;
            }
        }
    }

    private static TimeSpan CalculateBackoff(int attempt, TimeSpan baseDelay, TimeSpan maxDelay)
    {
        var delayMs = baseDelay.TotalMilliseconds * Math.Pow(2, attempt);
        var cappedMs = Math.Min(delayMs, maxDelay.TotalMilliseconds);
        return TimeSpan.FromMilliseconds(cappedMs);
    }
}

public static class ResiliencyExtensions
{
    public static ResiliencyDecorator Decorate(this ResiliencyPolicy policy)
        => new(policy);
}
