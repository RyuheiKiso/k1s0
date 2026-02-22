namespace K1s0.System.ServiceAuth;

public interface IServiceAuthClient : IAsyncDisposable
{
    Task<ServiceToken> GetTokenAsync(CancellationToken ct = default);

    Task<ServiceToken> GetCachedTokenAsync(CancellationToken ct = default);

    Task<SpiffeId> ValidateSpiffeIdAsync(string uri, string expectedNamespace, CancellationToken ct = default);
}
