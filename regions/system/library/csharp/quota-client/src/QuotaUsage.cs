namespace K1s0.System.QuotaClient;

public record QuotaUsage(
    string QuotaId,
    ulong Used,
    ulong Limit,
    QuotaPeriod Period,
    DateTimeOffset ResetAt);
