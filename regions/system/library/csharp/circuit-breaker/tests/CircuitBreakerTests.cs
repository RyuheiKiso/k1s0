using K1s0.System.CircuitBreaker;

namespace K1s0.System.CircuitBreaker.Tests;

public class CircuitBreakerTests
{
    [Fact]
    public void InitialState_IsClosed()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(3, 2, TimeSpan.FromSeconds(5)));
        Assert.Equal(CircuitState.Closed, cb.State);
        Assert.False(cb.IsOpen);
    }

    [Fact]
    public void RecordFailure_ReachesThreshold_Opens()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(2, 1, TimeSpan.FromSeconds(30)));
        cb.RecordFailure();
        Assert.Equal(CircuitState.Closed, cb.State);
        cb.RecordFailure();
        Assert.True(cb.IsOpen);
    }

    [Fact]
    public void RecordSuccess_ResetsFailureCount()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(3, 1, TimeSpan.FromSeconds(30)));
        cb.RecordFailure();
        cb.RecordFailure();
        cb.RecordSuccess();
        cb.RecordFailure();
        Assert.Equal(CircuitState.Closed, cb.State);
    }

    [Fact]
    public async Task CallAsync_WhenOpen_ThrowsCircuitBreakerOpenException()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(1, 1, TimeSpan.FromSeconds(30)));
        cb.RecordFailure();

        await Assert.ThrowsAsync<CircuitBreakerOpenException>(
            () => cb.CallAsync(() => Task.FromResult(42)));
    }

    [Fact]
    public async Task CallAsync_Success_ReturnsResult()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(3, 1, TimeSpan.FromSeconds(5)));
        var result = await cb.CallAsync(() => Task.FromResult(42));
        Assert.Equal(42, result);
    }

    [Fact]
    public async Task CallAsync_Failure_RecordsAndRethrows()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(2, 1, TimeSpan.FromSeconds(5)));

        await Assert.ThrowsAsync<InvalidOperationException>(
            () => cb.CallAsync<int>(() => throw new InvalidOperationException("boom")));

        await Assert.ThrowsAsync<InvalidOperationException>(
            () => cb.CallAsync<int>(() => throw new InvalidOperationException("boom")));

        Assert.True(cb.IsOpen);
    }

    [Fact]
    public void Timeout_TransitionsToHalfOpen()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(1, 1, TimeSpan.FromMilliseconds(1)));
        cb.RecordFailure();
        Assert.True(cb.IsOpen);

        Thread.Sleep(50);
        Assert.Equal(CircuitState.HalfOpen, cb.State);
    }

    [Fact]
    public void HalfOpen_SuccessThreshold_ClosesCB()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig(1, 2, TimeSpan.FromMilliseconds(1)));
        cb.RecordFailure();
        Thread.Sleep(50);

        Assert.Equal(CircuitState.HalfOpen, cb.State);
        cb.RecordSuccess();
        Assert.Equal(CircuitState.HalfOpen, cb.State);
        cb.RecordSuccess();
        Assert.Equal(CircuitState.Closed, cb.State);
    }
}
