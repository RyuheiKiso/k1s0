using Xunit;

namespace K1s0.System.Telemetry.Tests;

public class TelemetryInitializerTests
{
    [Fact]
    public void InitTracing_WithoutEndpoint_ReturnsNull()
    {
        var config = new TelemetryConfig
        {
            ServiceName = "test-service",
            Version = "1.0.0",
        };

        var provider = TelemetryInitializer.InitTracing(config);

        Assert.Null(provider);
    }

    [Fact]
    public void InitTracing_WithEndpoint_ReturnsProvider()
    {
        var config = new TelemetryConfig
        {
            ServiceName = "test-service",
            Version = "1.0.0",
            Endpoint = "http://localhost:4317",
            SampleRate = 1.0,
        };

        var provider = TelemetryInitializer.InitTracing(config);

        Assert.NotNull(provider);
        provider?.Dispose();
    }

    [Fact]
    public void CreateActivitySource_ReturnsNamedSource()
    {
        var source = TelemetryInitializer.CreateActivitySource("my-service");

        Assert.Equal("my-service", source.Name);
        source.Dispose();
    }
}
