namespace K1s0.System.QuotaClient;

public record QuotaClientConfig(
    string ServerUrl,
    TimeSpan? Timeout = null,
    TimeSpan? PolicyCacheTtl = null);
