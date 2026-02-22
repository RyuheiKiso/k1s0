using Microsoft.IdentityModel.Tokens;

namespace K1s0.System.Auth;

public interface IJwksFetcher
{
    Task<IReadOnlyList<JsonWebKey>> FetchKeysAsync(string jwksUrl, CancellationToken ct = default);
}
