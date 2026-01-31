using System.Text.Json;
using System.Text.Json.Serialization;

namespace K1s0.Auth.Oidc;

/// <summary>
/// Represents the OpenID Connect discovery document.
/// </summary>
/// <param name="Issuer">The issuer identifier.</param>
/// <param name="JwksUri">The URL of the JSON Web Key Set document.</param>
/// <param name="AuthorizationEndpoint">The authorization endpoint URL.</param>
/// <param name="TokenEndpoint">The token endpoint URL.</param>
/// <param name="UserinfoEndpoint">The userinfo endpoint URL.</param>
public record OidcConfiguration(
    [property: JsonPropertyName("issuer")] string Issuer,
    [property: JsonPropertyName("jwks_uri")] string JwksUri,
    [property: JsonPropertyName("authorization_endpoint")] string AuthorizationEndpoint,
    [property: JsonPropertyName("token_endpoint")] string TokenEndpoint,
    [property: JsonPropertyName("userinfo_endpoint")] string UserinfoEndpoint);

/// <summary>
/// Discovers OpenID Connect configuration from an issuer's well-known endpoint.
/// </summary>
public class OidcDiscovery
{
    private readonly string _issuerUrl;
    private readonly HttpClient _httpClient;
    private OidcConfiguration? _cached;

    /// <summary>
    /// Initializes a new instance of the <see cref="OidcDiscovery"/> class.
    /// </summary>
    /// <param name="issuerUrl">The OIDC issuer URL.</param>
    /// <param name="httpClient">Optional HTTP client.</param>
    public OidcDiscovery(string issuerUrl, HttpClient? httpClient = null)
    {
        _issuerUrl = issuerUrl.TrimEnd('/');
        _httpClient = httpClient ?? new HttpClient();
    }

    /// <summary>
    /// Fetches the OpenID Connect discovery document.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The OIDC configuration.</returns>
    /// <exception cref="DiscoveryException">Thrown when discovery fails.</exception>
    public async Task<OidcConfiguration> DiscoverAsync(CancellationToken ct = default)
    {
        if (_cached is not null)
        {
            return _cached;
        }

        var url = $"{_issuerUrl}/.well-known/openid-configuration";

        try
        {
            var response = await _httpClient.GetStringAsync(url, ct).ConfigureAwait(false);
            var config = JsonSerializer.Deserialize<OidcConfiguration>(response)
                ?? throw new DiscoveryException("Discovery document deserialized to null");

            _cached = config;
            return config;
        }
        catch (HttpRequestException ex)
        {
            throw new DiscoveryException($"Failed to fetch OIDC discovery from {url}", ex);
        }
        catch (JsonException ex)
        {
            throw new DiscoveryException($"Failed to parse OIDC discovery document from {url}", ex);
        }
    }
}
