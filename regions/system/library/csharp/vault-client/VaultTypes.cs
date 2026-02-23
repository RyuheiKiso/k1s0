namespace K1s0.System.VaultClient;

public record Secret(
    string Path,
    IReadOnlyDictionary<string, string> Data,
    long Version,
    DateTimeOffset CreatedAt);

public record SecretRotatedEvent(string Path, long Version);

public record VaultClientConfig(
    string ServerUrl,
    TimeSpan? CacheTtl = null,
    int CacheMaxCapacity = 500);
