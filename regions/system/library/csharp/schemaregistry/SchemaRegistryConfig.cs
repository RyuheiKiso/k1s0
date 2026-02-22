namespace K1s0.System.SchemaRegistry;

public sealed record SchemaRegistryConfig
{
    public required string Url { get; init; }

    public string? Username { get; init; }

    public string? Password { get; init; }

    public CompatibilityMode CompatibilityMode { get; init; } = CompatibilityMode.Backward;

    public static string SubjectName(string topic, string suffix = "value") => $"{topic}-{suffix}";
}
