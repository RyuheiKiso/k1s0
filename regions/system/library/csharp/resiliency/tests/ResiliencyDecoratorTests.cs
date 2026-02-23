using K1s0.System.Resiliency;
using Xunit;

namespace K1s0.System.Resiliency.Tests;

public class ResiliencyDecoratorTests
{
    [Fact]
    public async Task ExecuteAsync_Success()
    {
        var policy = new ResiliencyPolicy();
        using var decorator = new ResiliencyDecorator(policy);

        var result = await decorator.ExecuteAsync(ct => Task.FromResult(42));

        Assert.Equal(42, result);
    }

    [Fact]
    public async Task ExecuteAsync_RetrySuccess()
    {
        var policy = new ResiliencyPolicy
        {
            Retry = new RetryConfig
            {
                MaxAttempts = 3,
                BaseDelay = TimeSpan.FromMilliseconds(10),
                MaxDelay = TimeSpan.FromMilliseconds(100),
            },
        };
        using var decorator = new ResiliencyDecorator(policy);

        var counter = 0;
        var result = await decorator.ExecuteAsync(ct =>
        {
            counter++;
            if (counter < 3) throw new InvalidOperationException("fail");
            return Task.FromResult(99);
        });

        Assert.Equal(99, result);
        Assert.Equal(3, counter);
    }

    [Fact]
    public async Task ExecuteAsync_MaxRetriesExceeded()
    {
        var policy = new ResiliencyPolicy
        {
            Retry = new RetryConfig
            {
                MaxAttempts = 2,
                BaseDelay = TimeSpan.FromMilliseconds(1),
                MaxDelay = TimeSpan.FromMilliseconds(10),
            },
        };
        using var decorator = new ResiliencyDecorator(policy);

        var ex = await Assert.ThrowsAsync<ResiliencyException>(() =>
            decorator.ExecuteAsync<int>(ct =>
                throw new InvalidOperationException("always fail")));

        Assert.Equal(ResiliencyErrorKind.MaxRetriesExceeded, ex.Kind);
    }

    [Fact]
    public async Task ExecuteAsync_Timeout()
    {
        var policy = new ResiliencyPolicy { Timeout = TimeSpan.FromMilliseconds(50) };
        using var decorator = new ResiliencyDecorator(policy);

        var ex = await Assert.ThrowsAsync<ResiliencyException>(() =>
            decorator.ExecuteAsync(async ct =>
            {
                await Task.Delay(TimeSpan.FromSeconds(1), ct);
                return 42;
            }));

        Assert.Equal(ResiliencyErrorKind.Timeout, ex.Kind);
    }

    [Fact]
    public async Task ExecuteAsync_CircuitBreakerOpens()
    {
        var policy = new ResiliencyPolicy
        {
            CircuitBreaker = new CircuitBreakerConfig
            {
                FailureThreshold = 3,
                RecoveryTimeout = TimeSpan.FromMinutes(1),
                HalfOpenMaxCalls = 1,
            },
        };
        using var decorator = new ResiliencyDecorator(policy);

        for (var i = 0; i < 3; i++)
        {
            try
            {
                await decorator.ExecuteAsync<int>(ct =>
                    throw new InvalidOperationException("fail"));
            }
            catch (ResiliencyException)
            {
                // expected - max retries exceeded
            }
            catch (InvalidOperationException)
            {
                // expected
            }
        }

        var ex = await Assert.ThrowsAsync<ResiliencyException>(() =>
            decorator.ExecuteAsync(ct => Task.FromResult(42)));

        Assert.Equal(ResiliencyErrorKind.CircuitBreakerOpen, ex.Kind);
    }

    [Fact]
    public async Task ExecuteAsync_BulkheadFull()
    {
        var policy = new ResiliencyPolicy
        {
            Bulkhead = new BulkheadConfig
            {
                MaxConcurrentCalls = 1,
                MaxWaitDuration = TimeSpan.FromMilliseconds(50),
            },
        };
        using var decorator = new ResiliencyDecorator(policy);

        var tcs = new TaskCompletionSource<int>();

        // Occupy the single slot
        var longRunning = decorator.ExecuteAsync(ct => tcs.Task);

        await Task.Delay(10);

        var ex = await Assert.ThrowsAsync<ResiliencyException>(() =>
            decorator.ExecuteAsync(ct => Task.FromResult(2)));

        Assert.Equal(ResiliencyErrorKind.BulkheadFull, ex.Kind);

        tcs.SetResult(1);
        await longRunning;
    }

    [Fact]
    public void Decorate_Extension()
    {
        var policy = new ResiliencyPolicy();
        using var decorator = policy.Decorate();
        Assert.NotNull(decorator);
    }
}
