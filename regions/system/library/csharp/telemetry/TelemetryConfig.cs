namespace K1s0.System.Telemetry;

public sealed record TelemetryConfig
{
    public string ServiceName { get; init; } = string.Empty;

    public string Version { get; init; } = string.Empty;

    public string Tier { get; init; } = string.Empty;

    public string Environment { get; init; } = string.Empty;

    public string? Endpoint { get; init; }

    public double SampleRate { get; init; } = 1.0;

    public string LogLevel { get; init; } = "Information";

    public string LogFormat { get; init; } = "json";
}
