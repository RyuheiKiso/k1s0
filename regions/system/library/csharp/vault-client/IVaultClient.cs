namespace K1s0.System.VaultClient;

public interface IVaultClient
{
    Task<Secret> GetSecretAsync(string path, CancellationToken ct = default);
    Task<string> GetSecretValueAsync(string path, string key, CancellationToken ct = default);
    Task<IReadOnlyList<string>> ListSecretsAsync(string pathPrefix, CancellationToken ct = default);
    IAsyncEnumerable<SecretRotatedEvent> WatchSecretAsync(string path, CancellationToken ct = default);
}
