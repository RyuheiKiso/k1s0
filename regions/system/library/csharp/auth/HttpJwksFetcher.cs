using Microsoft.IdentityModel.Tokens;

namespace K1s0.System.Auth;

public sealed class HttpJwksFetcher : IJwksFetcher, IAsyncDisposable
{
    private readonly IHttpClientFactory _httpClientFactory;
    private bool _disposed;

    public HttpJwksFetcher(IHttpClientFactory httpClientFactory)
    {
        _httpClientFactory = httpClientFactory;
    }

    public async Task<IReadOnlyList<JsonWebKey>> FetchKeysAsync(string jwksUrl, CancellationToken ct = default)
    {
        ObjectDisposedException.ThrowIf(_disposed, this);

        var client = _httpClientFactory.CreateClient(nameof(HttpJwksFetcher));

        HttpResponseMessage response;
        try
        {
            response = await client.GetAsync(jwksUrl, ct).ConfigureAwait(false);
            response.EnsureSuccessStatusCode();
        }
        catch (HttpRequestException ex)
        {
            throw new AuthException("JWKS_FETCH_FAILED", $"Failed to fetch JWKS from {jwksUrl}", ex);
        }

        var json = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);

        JsonWebKeySet keySet;
        try
        {
            keySet = new JsonWebKeySet(json);
        }
        catch (Exception ex) when (ex is not AuthException and not OperationCanceledException)
        {
            throw new AuthException("JWKS_PARSE_FAILED", "Failed to parse JWKS response", ex);
        }

        return keySet.Keys.ToList().AsReadOnly();
    }

    public ValueTask DisposeAsync()
    {
        _disposed = true;
        return ValueTask.CompletedTask;
    }
}
