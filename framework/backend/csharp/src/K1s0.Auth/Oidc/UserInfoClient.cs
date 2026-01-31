using System.Net.Http.Headers;
using System.Text.Json;

namespace K1s0.Auth.Oidc;

/// <summary>
/// Client for fetching user information from an OIDC userinfo endpoint.
/// </summary>
public class UserInfoClient
{
    private readonly string _userinfoEndpoint;
    private readonly HttpClient _httpClient;

    /// <summary>
    /// Initializes a new instance of the <see cref="UserInfoClient"/> class.
    /// </summary>
    /// <param name="userinfoEndpoint">The userinfo endpoint URL.</param>
    /// <param name="httpClient">Optional HTTP client.</param>
    public UserInfoClient(string userinfoEndpoint, HttpClient? httpClient = null)
    {
        _userinfoEndpoint = userinfoEndpoint;
        _httpClient = httpClient ?? new HttpClient();
    }

    /// <summary>
    /// Fetches user information using the provided access token.
    /// </summary>
    /// <param name="accessToken">The OAuth2 access token.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The user information.</returns>
    /// <exception cref="AuthException">Thrown when the userinfo request fails.</exception>
    public async Task<UserInfo> GetUserInfoAsync(string accessToken, CancellationToken ct = default)
    {
        using var request = new HttpRequestMessage(HttpMethod.Get, _userinfoEndpoint);
        request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", accessToken);

        using var response = await _httpClient.SendAsync(request, ct).ConfigureAwait(false);

        if (!response.IsSuccessStatusCode)
        {
            throw new AuthException(
                $"Userinfo request failed with status {response.StatusCode}",
                "auth.userinfo_failed");
        }

        var content = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        return JsonSerializer.Deserialize<UserInfo>(content)
            ?? throw new AuthException("Userinfo response deserialized to null", "auth.userinfo_failed");
    }
}
