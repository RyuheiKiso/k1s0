using Serilog;
using Serilog.Events;
using Serilog.Formatting.Compact;

namespace K1s0.System.Telemetry;

public static class K1s0Logger
{
    public static ILogger NewLogger(TelemetryConfig config)
    {
        var loggerConfig = new LoggerConfiguration()
            .MinimumLevel.Is(ParseLogLevel(config.LogLevel))
            .Enrich.WithProperty("service", config.ServiceName)
            .Enrich.WithProperty("version", config.Version)
            .Enrich.WithProperty("tier", config.Tier)
            .Enrich.WithProperty("environment", config.Environment);

        if (string.Equals(config.LogFormat, "json", StringComparison.OrdinalIgnoreCase))
        {
            loggerConfig = loggerConfig.WriteTo.Console(new CompactJsonFormatter());
        }
        else
        {
            loggerConfig = loggerConfig.WriteTo.Console();
        }

        return loggerConfig.CreateLogger();
    }

    private static LogEventLevel ParseLogLevel(string level) => level.ToLowerInvariant() switch
    {
        "debug" or "verbose" => LogEventLevel.Debug,
        "information" or "info" => LogEventLevel.Information,
        "warning" or "warn" => LogEventLevel.Warning,
        "error" => LogEventLevel.Error,
        "fatal" => LogEventLevel.Fatal,
        _ => LogEventLevel.Information,
    };
}
