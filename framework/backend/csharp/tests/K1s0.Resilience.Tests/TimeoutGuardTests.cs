using FluentAssertions;
using Xunit;

namespace K1s0.Resilience.Tests;

public class TimeoutGuardTests
{
    [Fact]
    public async Task ExecuteAsync_CompletesBeforeTimeout_ReturnsResult()
    {
        var guard = new TimeoutGuard(new TimeoutConfig(DurationSeconds: 5.0));

        var result = await guard.ExecuteAsync(ct => Task.FromResult(42));

        result.Should().Be(42);
    }

    [Fact]
    public async Task ExecuteAsync_ExceedsTimeout_ThrowsK1s0TimeoutException()
    {
        var guard = new TimeoutGuard(new TimeoutConfig(DurationSeconds: 0.1));

        var act = () => guard.ExecuteAsync(async ct =>
        {
            await Task.Delay(TimeSpan.FromSeconds(5), ct);
            return 42;
        });

        await act.Should().ThrowAsync<K1s0TimeoutException>();
    }

    [Fact]
    public async Task ExecuteAsync_ExternalCancellation_ThrowsOperationCanceledException()
    {
        var guard = new TimeoutGuard(new TimeoutConfig(DurationSeconds: 30.0));
        using var cts = new CancellationTokenSource();
        cts.Cancel();

        var act = () => guard.ExecuteAsync(async ct =>
        {
            ct.ThrowIfCancellationRequested();
            await Task.Delay(1, ct);
            return 42;
        }, cts.Token);

        await act.Should().ThrowAsync<OperationCanceledException>();
    }

    [Fact]
    public void TimeoutConfig_InvalidDuration_ThrowsArgumentOutOfRangeException()
    {
        var act = () => new TimeoutConfig(DurationSeconds: 0.0);

        act.Should().Throw<ArgumentOutOfRangeException>();
    }

    [Fact]
    public void TimeoutConfig_ExceedsMaxDuration_ThrowsArgumentOutOfRangeException()
    {
        var act = () => new TimeoutConfig(DurationSeconds: 301.0);

        act.Should().Throw<ArgumentOutOfRangeException>();
    }
}
