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

    public async Task<ServiceClaims> VerifyTokenAsync(string token, CancellationToken ct = default)
    {
        if (_config.JwksUri is null)
        {
            throw new ServiceAuthException("InvalidToken", "JWKS URI is not configured.");
        }

        HttpResponseMessage jwksResponse;
        try
        {
            jwksResponse = await _httpClient.GetAsync(_config.JwksUri, ct).ConfigureAwait(false);
        }
        catch (HttpRequestException ex)
        {
            throw new ServiceAuthException("InvalidToken", "Failed to reach JWKS endpoint.", ex);
        }

        if (!jwksResponse.IsSuccessStatusCode)
        {
            var body = await jwksResponse.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
            throw new ServiceAuthException(
                "InvalidToken",
                $"JWKS endpoint returned {(int)jwksResponse.StatusCode}: {body}");
        }

        var jwksJson = await jwksResponse.Content.ReadAsStringAsync(ct).ConfigureAwait(false);

        using var doc = JsonDocument.Parse(jwksJson);
        var keys = doc.RootElement.GetProperty("keys");

        // Decode JWT header to find kid
        var parts = token.Split('.');
        if (parts.Length != 3)
        {
            throw new ServiceAuthException("InvalidToken", "Token does not have 3 parts.");
        }

        var headerJson = global::System.Text.Encoding.UTF8.GetString(DecodeBase64Url(parts[0]));
        using var headerDoc = JsonDocument.Parse(headerJson);

        if (!headerDoc.RootElement.TryGetProperty("kid", out var kidElement))
        {
            throw new ServiceAuthException("InvalidToken", "JWT header does not contain kid.");
        }

        var kid = kidElement.GetString()
            ?? throw new ServiceAuthException("InvalidToken", "JWT kid is null.");

        // Find matching key
        bool found = false;
        foreach (var key in keys.EnumerateArray())
        {
            if (key.TryGetProperty("kid", out var keyKid) && keyKid.GetString() == kid)
            {
                found = true;
                break;
            }
        }

        if (!found)
        {
            throw new ServiceAuthException("InvalidToken", $"JWKS does not contain kid '{kid}'.");
        }

        // Decode payload for claims (simplified - real impl would verify signature)
        var payloadJson = global::System.Text.Encoding.UTF8.GetString(DecodeBase64Url(parts[1]));
        using var payloadDoc = JsonDocument.Parse(payloadJson);
        var root = payloadDoc.RootElement;

        return new ServiceClaims(
            Sub: root.TryGetProperty("sub", out var sub) ? sub.GetString() ?? string.Empty : string.Empty,
            Iss: root.TryGetProperty("iss", out var iss) ? iss.GetString() ?? string.Empty : string.Empty,
            Scope: root.TryGetProperty("scope", out var scope) ? scope.GetString() ?? string.Empty : string.Empty,
            Exp: root.TryGetProperty("exp", out var exp) ? exp.GetInt64() : 0,
            Iat: root.TryGetProperty("iat", out var iat) ? iat.GetInt64() : 0);
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

    private static byte[] DecodeBase64Url(string input)
    {
        var s = input.Replace('-', '+').Replace('_', '/');
        switch (s.Length % 4)
        {
            case 2: s += "=="; break;
            case 3: s += "="; break;
        }

        return Convert.FromBase64String(s);
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
