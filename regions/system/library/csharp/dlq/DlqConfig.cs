namespace K1s0.System.Dlq;

public sealed record DlqConfig(
    string BaseUrl,
    int TimeoutSeconds = 30);
