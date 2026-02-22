using Serilog;
using Xunit;

namespace K1s0.System.Telemetry.Tests;

public class K1s0LoggerTests
{
    [Fact]
    public void NewLogger_JsonFormat_ReturnsLogger()
    {
        var config = new TelemetryConfig
        {
            ServiceName = "test",
            Version = "1.0",
            Tier = "system",
            Environment = "dev",
            LogLevel = "debug",
            LogFormat = "json",
        };

        ILogger logger = K1s0Logger.NewLogger(config);

        Assert.NotNull(logger);
    }

    [Fact]
    public void NewLogger_TextFormat_ReturnsLogger()
    {
        var config = new TelemetryConfig
        {
            ServiceName = "test",
            Version = "1.0",
            Tier = "system",
            Environment = "dev",
            LogLevel = "info",
            LogFormat = "text",
        };

        ILogger logger = K1s0Logger.NewLogger(config);

        Assert.NotNull(logger);
    }

    [Theory]
    [InlineData("debug")]
    [InlineData("info")]
    [InlineData("warn")]
    [InlineData("error")]
    [InlineData("fatal")]
    [InlineData("unknown")]
    public void NewLogger_AllLogLevels_DoNotThrow(string level)
    {
        var config = new TelemetryConfig
        {
            ServiceName = "test",
            Version = "1.0",
            Tier = "system",
            Environment = "dev",
            LogLevel = level,
            LogFormat = "json",
        };

        var exception = Record.Exception(() => K1s0Logger.NewLogger(config));

        Assert.Null(exception);
    }
}
