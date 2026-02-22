using System.Diagnostics.Metrics;
using Xunit;

namespace K1s0.System.Telemetry.Tests;

public class K1s0MetricsTests : IDisposable
{
    private readonly MeterFactory _meterFactory;
    private readonly K1s0Metrics _metrics;

    public K1s0MetricsTests()
    {
        _meterFactory = new MeterFactory();
        _metrics = new K1s0Metrics(_meterFactory);
    }

    public void Dispose()
    {
        _meterFactory.Dispose();
    }

    [Fact]
    public void RecordRequest_DoesNotThrow()
    {
        var exception = Record.Exception(
            () => _metrics.RecordRequest("GET", "/api/test", 200, 15.5));
        Assert.Null(exception);
    }

    [Fact]
    public void RecordRequest_ErrorStatus_DoesNotThrow()
    {
        var exception = Record.Exception(
            () => _metrics.RecordRequest("POST", "/api/test", 500, 100.0));
        Assert.Null(exception);
    }

    [Fact]
    public void IncrementDecrementInFlight_DoesNotThrow()
    {
        var exception = Record.Exception(() =>
        {
            _metrics.IncrementInFlight();
            _metrics.DecrementInFlight();
        });
        Assert.Null(exception);
    }
}

internal sealed class MeterFactory : IMeterFactory
{
    private readonly List<Meter> _meters = [];

    public Meter Create(MeterOptions options)
    {
        var meter = new Meter(options.Name, options.Version);
        _meters.Add(meter);
        return meter;
    }

    public void Dispose()
    {
        foreach (var meter in _meters)
        {
            meter.Dispose();
        }

        _meters.Clear();
    }
}
