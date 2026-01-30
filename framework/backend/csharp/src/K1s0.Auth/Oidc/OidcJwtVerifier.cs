using K1s0.Auth.Jwt;

namespace K1s0.Auth.Oidc;

/// <summary>
/// Verifies JWT tokens using auto-discovered OIDC configuration.
/// Combines <see cref="OidcDiscovery"/> with <see cref="JwtVerifier"/>.
/// </summary>
public class OidcJwtVerifier
{
    private readonly OidcDiscovery _discovery;
    private readonly string _audience;
    private readonly int _clockSkewSeconds;
    private readonly HttpClient _httpClient;
    private JwtVerifier? _verifier;
    private readonly SemaphoreSlim _initLock = new(1, 1);

    /// <summary>
    /// Initializes a new instance of the <see cref="OidcJwtVerifier"/> class.
    /// </summary>
    /// <param name="issuerUrl">The OIDC issuer URL.</param>
    /// <param name="audience">The expected audience.</param>
    /// <param name="clockSkewSeconds">Allowed clock skew in seconds.</param>
    /// <param name="httpClient">Optional HTTP client.</param>
    public OidcJwtVerifier(string issuerUrl, string audience, int clockSkewSeconds = 30, HttpClient? httpClient = null)
    {
        _httpClient = httpClient ?? new HttpClient();
        _discovery = new OidcDiscovery(issuerUrl, _httpClient);
        _audience = audience;
        _clockSkewSeconds = clockSkewSeconds;
    }

    /// <summary>
    /// Verifies a JWT token using auto-discovered OIDC configuration.
    /// </summary>
    /// <param name="token">The raw JWT token string.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The verified claims.</returns>
    public async Task<Claims> VerifyAsync(string token, CancellationToken ct = default)
    {
        var verifier = await GetVerifierAsync(ct).ConfigureAwait(false);
        return await verifier.VerifyAsync(token, ct).ConfigureAwait(false);
    }

    private async Task<JwtVerifier> GetVerifierAsync(CancellationToken ct)
    {
        if (_verifier is not null)
        {
            return _verifier;
        }

        await _initLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            if (_verifier is not null)
            {
                return _verifier;
            }

            var oidcConfig = await _discovery.DiscoverAsync(ct).ConfigureAwait(false);
            var config = new JwtVerifierConfig(
                Issuer: oidcConfig.Issuer,
                JwksUri: oidcConfig.JwksUri,
                Audience: _audience,
                ClockSkewSeconds: _clockSkewSeconds);

            _verifier = new JwtVerifier(config, _httpClient);
            return _verifier;
        }
        finally
        {
            _initLock.Release();
        }
    }
}
