using K1s0.System.Retry;

namespace K1s0.System.Retry.Tests;

public class RetryConfigTests
{
    [Fact]
    public void ComputeDelay_ExponentialGrowth_WithoutJitter()
    {
        var config = new RetryConfig(MaxAttempts: 5, InitialDelay: TimeSpan.FromMilliseconds(100), Jitter: false);

        var d0 = config.ComputeDelay(0);
        var d1 = config.ComputeDelay(1);
        var d2 = config.ComputeDelay(2);

        Assert.Equal(100, d0.TotalMilliseconds, precision: 1);
        Assert.Equal(200, d1.TotalMilliseconds, precision: 1);
        Assert.Equal(400, d2.TotalMilliseconds, precision: 1);
    }

    [Fact]
    public void ComputeDelay_RespectsMaxDelay()
    {
        var config = new RetryConfig(InitialDelay: TimeSpan.FromSeconds(1), MaxDelay: TimeSpan.FromSeconds(5), Jitter: false);

        var d10 = config.ComputeDelay(10);

        Assert.True(d10.TotalSeconds <= 5);
    }

    [Fact]
    public void DefaultConfig_HasReasonableDefaults()
    {
        var config = new RetryConfig();

        Assert.Equal(3, config.MaxAttempts);
        Assert.Equal(2.0, config.Multiplier);
        Assert.True(config.Jitter);
    }
}

public class RetryPolicyTests
{
    [Fact]
    public async Task WithRetryAsync_SucceedsOnFirstAttempt()
    {
        var config = new RetryConfig(MaxAttempts: 3, InitialDelay: TimeSpan.FromMilliseconds(1), Jitter: false);
        var result = await RetryPolicy.WithRetryAsync(config, () => Task.FromResult(42));
        Assert.Equal(42, result);
    }

    [Fact]
    public async Task WithRetryAsync_RetriesAndSucceeds()
    {
        var config = new RetryConfig(MaxAttempts: 3, InitialDelay: TimeSpan.FromMilliseconds(1), Jitter: false);
        int attempt = 0;

        var result = await RetryPolicy.WithRetryAsync(config, () =>
        {
            attempt++;
            if (attempt < 3)
            {
                throw new InvalidOperationException("fail");
            }

            return Task.FromResult("ok");
        });

        Assert.Equal("ok", result);
        Assert.Equal(3, attempt);
    }

    [Fact]
    public async Task WithRetryAsync_ThrowsRetryExhausted()
    {
        var config = new RetryConfig(MaxAttempts: 2, InitialDelay: TimeSpan.FromMilliseconds(1), Jitter: false);

        var ex = await Assert.ThrowsAsync<RetryExhaustedException>(() =>
            RetryPolicy.WithRetryAsync<int>(config, () => throw new InvalidOperationException("always fail")));

        Assert.Equal(2, ex.Attempts);
        Assert.IsType<InvalidOperationException>(ex.LastError);
    }

    [Fact]
    public async Task WithRetryAsync_VoidVersion_Works()
    {
        var config = new RetryConfig(MaxAttempts: 2, InitialDelay: TimeSpan.FromMilliseconds(1), Jitter: false);
        int called = 0;

        await RetryPolicy.WithRetryAsync(config, () =>
        {
            called++;
            return Task.CompletedTask;
        });

        Assert.Equal(1, called);
    }
}

public class CircuitBreakerTests
{
    [Fact]
    public void InitialState_IsClosed()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig());
        Assert.Equal(CircuitBreakerState.Closed, cb.State);
    }

    [Fact]
    public async Task OpensAfterFailureThreshold()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig { FailureThreshold = 2 });

        await cb.RecordFailureAsync();
        Assert.Equal(CircuitBreakerState.Closed, cb.State);

        await cb.RecordFailureAsync();
        Assert.Equal(CircuitBreakerState.Open, cb.State);
        Assert.True(cb.IsOpen());
    }

    [Fact]
    public async Task SuccessResetsFailureCount()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig { FailureThreshold = 3 });

        await cb.RecordFailureAsync();
        await cb.RecordFailureAsync();
        await cb.RecordSuccessAsync();

        await cb.RecordFailureAsync();
        Assert.Equal(CircuitBreakerState.Closed, cb.State);
    }

    [Fact]
    public async Task HalfOpen_ClosesAfterSuccessThreshold()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig
        {
            FailureThreshold = 1,
            SuccessThreshold = 2,
            Timeout = TimeSpan.FromMilliseconds(50),
        });

        await cb.RecordFailureAsync();
        Assert.True(cb.IsOpen());

        await Task.Delay(100);
        Assert.False(cb.IsOpen());
        Assert.Equal(CircuitBreakerState.HalfOpen, cb.State);

        await cb.RecordSuccessAsync();
        Assert.Equal(CircuitBreakerState.HalfOpen, cb.State);

        await cb.RecordSuccessAsync();
        Assert.Equal(CircuitBreakerState.Closed, cb.State);
    }

    [Fact]
    public async Task HalfOpen_ReopensOnFailure()
    {
        var cb = new CircuitBreaker(new CircuitBreakerConfig
        {
            FailureThreshold = 1,
            Timeout = TimeSpan.FromMilliseconds(50),
        });

        await cb.RecordFailureAsync();
        await Task.Delay(100);
        Assert.False(cb.IsOpen());
        Assert.Equal(CircuitBreakerState.HalfOpen, cb.State);

        await cb.RecordFailureAsync();
        Assert.Equal(CircuitBreakerState.Open, cb.State);
    }
}
