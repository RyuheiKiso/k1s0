namespace K1s0.System.Dlq;

public sealed record DlqMessage(
    Guid Id,
    string OriginalTopic,
    string ErrorMessage,
    int RetryCount,
    int MaxRetries,
    string Payload,
    DlqStatus Status,
    DateTimeOffset CreatedAt,
    DateTimeOffset UpdatedAt);
