namespace K1s0.System.QuotaClient;

public record QuotaPolicy(
    string QuotaId,
    ulong Limit,
    QuotaPeriod Period,
    string ResetStrategy);
