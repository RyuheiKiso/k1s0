using K1s0.System.VaultClient;

namespace K1s0.System.VaultClient.Tests;

public class InMemoryVaultClientTests
{
    private static VaultClientConfig MakeConfig() =>
        new("http://localhost:8080", TimeSpan.FromMinutes(10), 500);

    private static Secret MakeSecret(string path) =>
        new(path,
            new Dictionary<string, string> { ["password"] = "s3cr3t", ["username"] = "admin" },
            1,
            DateTimeOffset.UtcNow);

    [Fact]
    public async Task GetSecret_ReturnsSecretWhenFound()
    {
        var client = new InMemoryVaultClient(MakeConfig());
        client.PutSecret(MakeSecret("system/db/primary"));
        var secret = await client.GetSecretAsync("system/db/primary");
        Assert.Equal("system/db/primary", secret.Path);
        Assert.Equal("s3cr3t", secret.Data["password"]);
    }

    [Fact]
    public async Task GetSecret_ThrowsWhenNotFound()
    {
        var client = new InMemoryVaultClient(MakeConfig());
        var ex = await Assert.ThrowsAsync<VaultException>(() => client.GetSecretAsync("missing/path"));
        Assert.Equal(VaultErrorCode.NotFound, ex.Code);
    }

    [Fact]
    public async Task GetSecretValue_ReturnsValue()
    {
        var client = new InMemoryVaultClient(MakeConfig());
        client.PutSecret(MakeSecret("system/db"));
        var value = await client.GetSecretValueAsync("system/db", "password");
        Assert.Equal("s3cr3t", value);
    }

    [Fact]
    public async Task GetSecretValue_ThrowsWhenKeyNotFound()
    {
        var client = new InMemoryVaultClient(MakeConfig());
        client.PutSecret(MakeSecret("system/db"));
        var ex = await Assert.ThrowsAsync<VaultException>(
            () => client.GetSecretValueAsync("system/db", "missing_key"));
        Assert.Equal(VaultErrorCode.NotFound, ex.Code);
    }

    [Fact]
    public async Task ListSecrets_ReturnsMatchingPaths()
    {
        var client = new InMemoryVaultClient(MakeConfig());
        client.PutSecret(MakeSecret("system/db/primary"));
        client.PutSecret(MakeSecret("system/db/replica"));
        client.PutSecret(MakeSecret("business/api/key"));
        var paths = await client.ListSecretsAsync("system/");
        Assert.Equal(2, paths.Count);
        Assert.All(paths, p => Assert.StartsWith("system/", p));
    }

    [Fact]
    public async Task ListSecrets_ReturnsEmptyWhenNoMatch()
    {
        var client = new InMemoryVaultClient(MakeConfig());
        var paths = await client.ListSecretsAsync("nothing/");
        Assert.Empty(paths);
    }

    [Fact]
    public async Task WatchSecret_ReturnsEmptyStream()
    {
        var client = new InMemoryVaultClient(MakeConfig());
        var events = new List<SecretRotatedEvent>();
        await foreach (var e in client.WatchSecretAsync("system/db"))
        {
            events.Add(e);
        }
        Assert.Empty(events);
    }

    [Fact]
    public void VaultException_HasCodeAndMessage()
    {
        var ex = new VaultException(VaultErrorCode.PermissionDenied, "secret/path");
        Assert.Equal(VaultErrorCode.PermissionDenied, ex.Code);
        Assert.Equal("secret/path", ex.Message);
    }

    [Fact]
    public void Config_StoresValues()
    {
        var config = MakeConfig();
        var client = new InMemoryVaultClient(config);
        Assert.Equal("http://localhost:8080", client.Config.ServerUrl);
        Assert.Equal(TimeSpan.FromMinutes(10), client.Config.CacheTtl);
        Assert.Equal(500, client.Config.CacheMaxCapacity);
    }

    [Fact]
    public void Secret_Record_Equality()
    {
        var data = new Dictionary<string, string> { ["key"] = "value" };
        var s1 = new Secret("path", data, 1, DateTimeOffset.UtcNow);
        Assert.Equal("path", s1.Path);
        Assert.Equal(1, s1.Version);
    }

    [Fact]
    public void SecretRotatedEvent_HasFields()
    {
        var evt = new SecretRotatedEvent("system/db", 2);
        Assert.Equal("system/db", evt.Path);
        Assert.Equal(2, evt.Version);
    }
}
