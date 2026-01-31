using FluentAssertions;
using Xunit;

namespace K1s0.Resilience.Tests;

public class ConcurrencyLimiterTests
{
    [Fact]
    public async Task ExecuteAsync_WithinLimit_ReturnsResult()
    {
        using var limiter = new ConcurrencyLimiter(new ConcurrencyConfig(MaxConcurrent: 2));

        var result = await limiter.ExecuteAsync(() => Task.FromResult(42));

        result.Should().Be(42);
    }

    [Fact]
    public async Task ExecuteAsync_ExceedsLimit_ThrowsConcurrencyLimitException()
    {
        using var limiter = new ConcurrencyLimiter(new ConcurrencyConfig(MaxConcurrent: 1));
        var blockingTcs = new TaskCompletionSource<int>();

        // Occupy the single slot.
        var occupyingTask = limiter.ExecuteAsync(() => blockingTcs.Task);

        // Second call should be rejected.
        var act = () => limiter.ExecuteAsync(() => Task.FromResult(99));

        await act.Should().ThrowAsync<ConcurrencyLimitException>();

        // Release the blocking task.
        blockingTcs.SetResult(1);
        await occupyingTask;
    }

    [Fact]
    public async Task Metrics_TracksActiveAndRejected()
    {
        using var limiter = new ConcurrencyLimiter(new ConcurrencyConfig(MaxConcurrent: 1));
        var blockingTcs = new TaskCompletionSource<int>();

        var occupyingTask = limiter.ExecuteAsync(() => blockingTcs.Task);
        limiter.Metrics.ActiveCount.Should().Be(1);

        try
        {
            await limiter.ExecuteAsync(() => Task.FromResult(0));
        }
        catch (ConcurrencyLimitException)
        {
            // Expected.
        }

        limiter.Metrics.RejectedCount.Should().Be(1);

        blockingTcs.SetResult(1);
        await occupyingTask;

        limiter.Metrics.ActiveCount.Should().Be(0);
    }

    [Fact]
    public async Task ExecuteAsync_AfterSlotFreed_Succeeds()
    {
        using var limiter = new ConcurrencyLimiter(new ConcurrencyConfig(MaxConcurrent: 1));

        await limiter.ExecuteAsync(() => Task.FromResult(1));
        var result = await limiter.ExecuteAsync(() => Task.FromResult(2));

        result.Should().Be(2);
    }

    [Fact]
    public void ConcurrencyConfig_InvalidMaxConcurrent_ThrowsArgumentOutOfRangeException()
    {
        var act = () => new ConcurrencyConfig(MaxConcurrent: 0);

        act.Should().Throw<ArgumentOutOfRangeException>();
    }
}
