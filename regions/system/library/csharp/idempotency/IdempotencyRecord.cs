namespace K1s0.System.Idempotency;

public enum IdempotencyStatus
{
    Pending,
    Completed,
    Failed,
}

public record IdempotencyRecord(
    string Key,
    IdempotencyStatus Status = IdempotencyStatus.Pending,
    string? ResponseBody = null,
    int? StatusCode = null,
    DateTimeOffset? CreatedAt = null,
    DateTimeOffset? ExpiresAt = null,
    DateTimeOffset? CompletedAt = null)
{
    public bool IsExpired() => ExpiresAt.HasValue && ExpiresAt.Value <= DateTimeOffset.UtcNow;
}
