using System.Net.Http.Json;
using System.Text.Json.Serialization;

namespace K1s0.System.Auth;

public sealed class DeviceFlowClient
{
    private readonly IHttpClientFactory _httpClientFactory;
    private readonly string _tokenEndpoint;
    private readonly string _deviceAuthorizationEndpoint;
    private readonly string _clientId;

    public DeviceFlowClient(
        IHttpClientFactory httpClientFactory,
        string deviceAuthorizationEndpoint,
        string tokenEndpoint,
        string clientId)
    {
        _httpClientFactory = httpClientFactory;
        _deviceAuthorizationEndpoint = deviceAuthorizationEndpoint;
        _tokenEndpoint = tokenEndpoint;
        _clientId = clientId;
    }

    public async Task<DeviceAuthorizationResponse> RequestDeviceAuthorizationAsync(CancellationToken ct = default)
    {
        var client = _httpClientFactory.CreateClient(nameof(DeviceFlowClient));

        var content = new FormUrlEncodedContent(new Dictionary<string, string>
        {
            ["client_id"] = _clientId,
            ["scope"] = "openid profile email",
        });

        var response = await client.PostAsync(_deviceAuthorizationEndpoint, content, ct).ConfigureAwait(false);
        response.EnsureSuccessStatusCode();

        return await response.Content.ReadFromJsonAsync<DeviceAuthorizationResponse>(ct).ConfigureAwait(false)
            ?? throw new AuthException("DEVICE_FLOW_FAILED", "Failed to parse device authorization response");
    }

    public async Task<DeviceTokenResponse> PollForTokenAsync(
        string deviceCode,
        int intervalSeconds = 5,
        CancellationToken ct = default)
    {
        var client = _httpClientFactory.CreateClient(nameof(DeviceFlowClient));

        while (!ct.IsCancellationRequested)
        {
            await Task.Delay(TimeSpan.FromSeconds(intervalSeconds), ct).ConfigureAwait(false);

            var content = new FormUrlEncodedContent(new Dictionary<string, string>
            {
                ["grant_type"] = "urn:ietf:params:oauth:grant-type:device_code",
                ["client_id"] = _clientId,
                ["device_code"] = deviceCode,
            });

            var response = await client.PostAsync(_tokenEndpoint, content, ct).ConfigureAwait(false);

            if (response.IsSuccessStatusCode)
            {
                return await response.Content.ReadFromJsonAsync<DeviceTokenResponse>(ct).ConfigureAwait(false)
                    ?? throw new AuthException("DEVICE_FLOW_FAILED", "Failed to parse token response");
            }

            var errorBody = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
            if (errorBody.Contains("authorization_pending", StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            if (errorBody.Contains("slow_down", StringComparison.OrdinalIgnoreCase))
            {
                intervalSeconds += 5;
                continue;
            }

            throw new AuthException("DEVICE_FLOW_FAILED", $"Device flow token request failed: {errorBody}");
        }

        throw new OperationCanceledException(ct);
    }
}

public sealed record DeviceAuthorizationResponse
{
    [JsonPropertyName("device_code")]
    public required string DeviceCode { get; init; }

    [JsonPropertyName("user_code")]
    public required string UserCode { get; init; }

    [JsonPropertyName("verification_uri")]
    public required string VerificationUri { get; init; }

    [JsonPropertyName("verification_uri_complete")]
    public string? VerificationUriComplete { get; init; }

    [JsonPropertyName("expires_in")]
    public int ExpiresIn { get; init; }

    [JsonPropertyName("interval")]
    public int Interval { get; init; } = 5;
}

public sealed record DeviceTokenResponse
{
    [JsonPropertyName("access_token")]
    public required string AccessToken { get; init; }

    [JsonPropertyName("token_type")]
    public required string TokenType { get; init; }

    [JsonPropertyName("expires_in")]
    public int ExpiresIn { get; init; }

    [JsonPropertyName("refresh_token")]
    public string? RefreshToken { get; init; }

    [JsonPropertyName("id_token")]
    public string? IdToken { get; init; }
}
