using FluentAssertions;
using Xunit;

namespace K1s0.Cache.Tests;

public class CacheMetricsTests
{
    [Fact]
    public void HitRate_ReturnsZero_WhenNoRecords()
    {
        var metrics = new CacheMetrics();

        metrics.HitRate.Should().Be(0.0);
    }

    [Fact]
    public void HitRate_ReturnsOne_WhenAllHits()
    {
        var metrics = new CacheMetrics();
        metrics.RecordHit();
        metrics.RecordHit();
        metrics.RecordHit();

        metrics.HitRate.Should().Be(1.0);
        metrics.HitCount.Should().Be(3);
        metrics.OperationCount.Should().Be(3);
    }

    [Fact]
    public void HitRate_ReturnsCorrectRatio()
    {
        var metrics = new CacheMetrics();
        metrics.RecordHit();
        metrics.RecordHit();
        metrics.RecordMiss();
        metrics.RecordMiss();

        metrics.HitRate.Should().BeApproximately(0.5, 0.001);
        metrics.HitCount.Should().Be(2);
        metrics.MissCount.Should().Be(2);
        metrics.OperationCount.Should().Be(4);
    }

    [Fact]
    public void HitRate_ReturnsZero_WhenAllMisses()
    {
        var metrics = new CacheMetrics();
        metrics.RecordMiss();
        metrics.RecordMiss();

        metrics.HitRate.Should().Be(0.0);
    }

    [Fact]
    public void Counters_AreThreadSafe()
    {
        var metrics = new CacheMetrics();
        const int iterations = 10_000;

        Parallel.For(0, iterations, _ => metrics.RecordHit());
        Parallel.For(0, iterations, _ => metrics.RecordMiss());

        metrics.HitCount.Should().Be(iterations);
        metrics.MissCount.Should().Be(iterations);
        metrics.OperationCount.Should().Be(iterations * 2);
    }
}
