using FluentAssertions;
using Moq;
using Xunit;

namespace K1s0.Cache.Tests;

public class WriteBehindTests
{
    private readonly Mock<ICacheOperations> _cacheMock;

    public WriteBehindTests()
    {
        _cacheMock = new Mock<ICacheOperations>();
    }

    [Fact]
    public async Task WriteAsync_BuffersAndWritesToCache()
    {
        var flushed = new List<KeyValuePair<string, string>>();
        await using var wb = new Patterns.WriteBehind(
            _cacheMock.Object,
            batch => { flushed.AddRange(batch); return Task.CompletedTask; },
            TimeSpan.FromSeconds(60));

        await wb.WriteAsync("k1", "v1");
        await wb.WriteAsync("k2", "v2");

        wb.TotalWrites.Should().Be(2);
        _cacheMock.Verify(c => c.SetAsync("k1", "v1", It.IsAny<TimeSpan?>(), It.IsAny<CancellationToken>()), Times.Once);
        _cacheMock.Verify(c => c.SetAsync("k2", "v2", It.IsAny<TimeSpan?>(), It.IsAny<CancellationToken>()), Times.Once);

        await wb.FlushAsync();

        flushed.Should().HaveCount(2);
        wb.TotalFlushes.Should().Be(1);
    }

    [Fact]
    public async Task FlushAsync_DoesNothing_WhenBufferEmpty()
    {
        var flushCount = 0;
        await using var wb = new Patterns.WriteBehind(
            _cacheMock.Object,
            _ => { flushCount++; return Task.CompletedTask; },
            TimeSpan.FromSeconds(60));

        await wb.FlushAsync();

        flushCount.Should().Be(0);
        wb.TotalFlushes.Should().Be(0);
    }

    [Fact]
    public async Task FlushAsync_IncrementsFailures_OnError()
    {
        await using var wb = new Patterns.WriteBehind(
            _cacheMock.Object,
            _ => throw new InvalidOperationException("flush error"),
            TimeSpan.FromSeconds(60));

        await wb.WriteAsync("k1", "v1");

        var act = () => wb.FlushAsync();
        await act.Should().ThrowAsync<InvalidOperationException>();

        wb.TotalFailures.Should().Be(1);
        wb.TotalFlushes.Should().Be(0);
    }

    [Fact]
    public async Task StartAndStop_RunsBackgroundFlush()
    {
        var flushCount = 0;
        await using var wb = new Patterns.WriteBehind(
            _cacheMock.Object,
            _ => { flushCount++; return Task.CompletedTask; },
            TimeSpan.FromMilliseconds(50));

        await wb.WriteAsync("k1", "v1");
        wb.StartAsync();

        // Wait for at least one flush cycle.
        await Task.Delay(200);

        await wb.StopAsync();

        // At least one flush should have occurred (background + final flush in StopAsync).
        wb.TotalFlushes.Should().BeGreaterThanOrEqualTo(1);
    }
}
