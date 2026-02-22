using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace K1s0.System.ServiceAuth;

public sealed class ServiceAuthClient : IServiceAuthClient
{
    private readonly HttpClient _httpClient;
    private readonly ServiceAuthConfig _config;
    private readonly SemaphoreSlim _semaphore = new(1, 1);
    private ServiceToken? _cachedToken;

    public ServiceAuthClient(HttpClient httpClient, ServiceAuthConfig config)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
        _config = config ?? throw new ArgumentNullException(nameof(config));
    }

    public async Task<ServiceToken> GetTokenAsync(CancellationToken ct = default)
    {
        var parameters = new Dictionary<string, string>
        {
            ["grant_type"] = "client_credentials",
            ["client_id"] = _config.ClientId,
            ["client_secret"] = _config.ClientSecret,
        };

        if (_config.Scopes is { Length: > 0 })
        {
            parameters["scope"] = string.Join(" ", _config.Scopes);
        }

        using var content = new FormUrlEncodedContent(parameters);
        HttpResponseMessage response;

        try
        {
            response = await _httpClient.PostAsync(_config.TokenUrl, content, ct).ConfigureAwait(false);
        }
        catch (HttpRequestException ex)
        {
            throw new ServiceAuthException("TokenFetch", "Failed to reach token endpoint.", ex);
        }

        if (!response.IsSuccessStatusCode)
        {
            var body = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
            throw new ServiceAuthException(
                "TokenFetch",
                $"Token endpoint returned {(int)response.StatusCode}: {body}");
        }

        var tokenResponse = await response.Content
            .ReadFromJsonAsync<TokenResponse>(_jsonOptions, ct)
            .ConfigureAwait(false);

        if (tokenResponse is null || string.IsNullOrEmpty(tokenResponse.AccessToken))
        {
            throw new ServiceAuthException("TokenFetch", "Token response is missing access_token.");
        }

        var expiresAt = DateTimeOffset.UtcNow.AddSeconds(tokenResponse.ExpiresIn);

        return new ServiceToken(
            tokenResponse.AccessToken,
            expiresAt,
            tokenResponse.TokenType ?? "Bearer");
    }

    public async Task<ServiceToken> GetCachedTokenAsync(CancellationToken ct = default)
    {
        var current = _cachedToken;
        if (current is not null && !current.IsNearExpiry)
        {
            return current;
        }

        await _semaphore.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            // Double-check after acquiring the lock.
            if (_cachedToken is not null && !_cachedToken.IsNearExpiry)
            {
                return _cachedToken;
            }

            _cachedToken = await GetTokenAsync(ct).ConfigureAwait(false);
            return _cachedToken;
        }
        finally
        {
            _semaphore.Release();
        }
    }

    public Task<SpiffeId> ValidateSpiffeIdAsync(
        string uri,
        string expectedNamespace,
        CancellationToken ct = default)
    {
        var spiffeId = SpiffeId.Parse(uri);

        if (!string.Equals(spiffeId.Namespace, expectedNamespace, StringComparison.Ordinal))
        {
            throw new ServiceAuthException(
                "InvalidSpiffeId",
                $"Expected namespace '{expectedNamespace}', got '{spiffeId.Namespace}'.");
        }

        return Task.FromResult(spiffeId);
    }

    public ValueTask DisposeAsync()
    {
        _semaphore.Dispose();
        return ValueTask.CompletedTask;
    }

    private static readonly JsonSerializerOptions _jsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
    };

    private sealed class TokenResponse
    {
        [JsonPropertyName("access_token")]
        public string? AccessToken { get; set; }

        [JsonPropertyName("expires_in")]
        public int ExpiresIn { get; set; }

        [JsonPropertyName("token_type")]
        public string? TokenType { get; set; }
    }
}
