namespace K1s0.System.QuotaClient;

public record QuotaStatus(
    bool Allowed,
    ulong Remaining,
    ulong Limit,
    DateTimeOffset ResetAt);
