namespace K1s0.System.Outbox;

public sealed record OutboxConfig(
    string ConnectionString,
    TimeSpan PollingInterval = default,
    int MaxRetries = 5,
    TimeSpan BackoffBase = default)
{
    public TimeSpan PollingInterval { get; init; } =
        PollingInterval == default ? TimeSpan.FromSeconds(5) : PollingInterval;

    public TimeSpan BackoffBase { get; init; } =
        BackoffBase == default ? TimeSpan.FromSeconds(1) : BackoffBase;
}
