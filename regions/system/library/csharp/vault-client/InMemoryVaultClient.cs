using System.Runtime.CompilerServices;

namespace K1s0.System.VaultClient;

public class InMemoryVaultClient : IVaultClient
{
    private readonly Dictionary<string, Secret> _store = new();
    private readonly VaultClientConfig _config;

    public InMemoryVaultClient(VaultClientConfig config)
    {
        _config = config;
    }

    public VaultClientConfig Config => _config;

    public void PutSecret(Secret secret)
    {
        _store[secret.Path] = secret;
    }

    public Task<Secret> GetSecretAsync(string path, CancellationToken ct = default)
    {
        if (!_store.TryGetValue(path, out var secret))
        {
            throw new VaultException(VaultErrorCode.NotFound, path);
        }
        return Task.FromResult(secret);
    }

    public async Task<string> GetSecretValueAsync(string path, string key, CancellationToken ct = default)
    {
        var secret = await GetSecretAsync(path, ct);
        if (!secret.Data.TryGetValue(key, out var value))
        {
            throw new VaultException(VaultErrorCode.NotFound, $"{path}/{key}");
        }
        return value;
    }

    public Task<IReadOnlyList<string>> ListSecretsAsync(string pathPrefix, CancellationToken ct = default)
    {
        var paths = _store.Keys.Where(k => k.StartsWith(pathPrefix)).ToList();
        return Task.FromResult<IReadOnlyList<string>>(paths.AsReadOnly());
    }

    public async IAsyncEnumerable<SecretRotatedEvent> WatchSecretAsync(
        string path,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        await Task.CompletedTask;
        yield break;
    }
}
