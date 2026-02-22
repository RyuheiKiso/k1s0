namespace K1s0.System.Outbox;

public sealed record OutboxMessage(
    Guid Id,
    string Topic,
    byte[] Payload,
    OutboxStatus Status,
    int RetryCount,
    DateTimeOffset CreatedAt,
    DateTimeOffset UpdatedAt,
    string? LastError);
