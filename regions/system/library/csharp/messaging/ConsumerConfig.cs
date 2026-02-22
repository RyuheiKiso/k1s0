namespace K1s0.System.Messaging;

public sealed record ConsumerConfig(
    string GroupId,
    string AutoOffsetReset = "earliest",
    bool EnableAutoCommit = false,
    int? MaxPollIntervalMs = null,
    int? SessionTimeoutMs = null);
