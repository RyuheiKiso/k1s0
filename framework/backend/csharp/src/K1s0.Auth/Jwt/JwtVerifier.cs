using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using Microsoft.IdentityModel.Tokens;
using System.Text.Json;

namespace K1s0.Auth.Jwt;

/// <summary>
/// Verifies JWT tokens using JWKS-based signature validation.
/// </summary>
public class JwtVerifier
{
    private readonly JwtVerifierConfig _config;
    private readonly HttpClient _httpClient;
    private readonly SemaphoreSlim _jwksLock = new(1, 1);
    private JsonWebKeySet? _cachedJwks;
    private DateTime _jwksCacheExpiry = DateTime.MinValue;
    private static readonly TimeSpan JwksCacheDuration = TimeSpan.FromHours(1);

    /// <summary>
    /// Initializes a new instance of the <see cref="JwtVerifier"/> class.
    /// </summary>
    /// <param name="config">The JWT verification configuration.</param>
    /// <param name="httpClient">Optional HTTP client for JWKS fetching. A default client is created if not provided.</param>
    public JwtVerifier(JwtVerifierConfig config, HttpClient? httpClient = null)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
        _httpClient = httpClient ?? new HttpClient();
    }

    /// <summary>
    /// Verifies a JWT token and extracts claims.
    /// </summary>
    /// <param name="token">The raw JWT token string.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The verified claims from the token.</returns>
    /// <exception cref="TokenExpiredException">Thrown when the token has expired.</exception>
    /// <exception cref="TokenInvalidException">Thrown when the token is invalid.</exception>
    public async Task<Claims> VerifyAsync(string token, CancellationToken ct = default)
    {
        var signingKeys = await GetSigningKeysAsync(ct).ConfigureAwait(false);

        var validationParameters = new TokenValidationParameters
        {
            ValidIssuer = _config.Issuer,
            ValidAudience = _config.Audience,
            IssuerSigningKeys = signingKeys,
            ValidAlgorithms = _config.Algorithms,
            ClockSkew = TimeSpan.FromSeconds(_config.ClockSkewSeconds),
            ValidateIssuer = true,
            ValidateAudience = true,
            ValidateLifetime = true,
            ValidateIssuerSigningKey = true,
        };

        var handler = new JwtSecurityTokenHandler
        {
            InboundClaimTypeMap = { },
        };
        handler.InboundClaimTypeMap.Clear();

        try
        {
            var principal = handler.ValidateToken(token, validationParameters, out var validatedToken);
            return ExtractClaims(principal);
        }
        catch (SecurityTokenExpiredException ex)
        {
            throw new TokenExpiredException($"Token expired at {ex.Expires:O}");
        }
        catch (SecurityTokenException ex)
        {
            throw new TokenInvalidException($"Token validation failed: {ex.Message}", ex);
        }
    }

    private async Task<IEnumerable<SecurityKey>> GetSigningKeysAsync(CancellationToken ct)
    {
        if (_cachedJwks is not null && DateTime.UtcNow < _jwksCacheExpiry)
        {
            return _cachedJwks.GetSigningKeys();
        }

        await _jwksLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            if (_cachedJwks is not null && DateTime.UtcNow < _jwksCacheExpiry)
            {
                return _cachedJwks.GetSigningKeys();
            }

            var response = await _httpClient.GetStringAsync(_config.JwksUri, ct).ConfigureAwait(false);
            _cachedJwks = new JsonWebKeySet(response);
            _jwksCacheExpiry = DateTime.UtcNow.Add(JwksCacheDuration);
            return _cachedJwks.GetSigningKeys();
        }
        catch (HttpRequestException ex)
        {
            throw new DiscoveryException($"Failed to fetch JWKS from {_config.JwksUri}", ex);
        }
        finally
        {
            _jwksLock.Release();
        }
    }

    private static Claims ExtractClaims(ClaimsPrincipal principal)
    {
        var sub = principal.FindFirstValue("sub") ?? string.Empty;
        var roles = GetClaimValues(principal, "roles");
        var permissions = GetClaimValues(principal, "permissions");
        var groups = GetClaimValues(principal, "groups");
        var tenantId = principal.FindFirstValue("tenant_id");

        var knownTypes = new HashSet<string>(StringComparer.OrdinalIgnoreCase)
        {
            "sub", "roles", "permissions", "groups", "tenant_id",
            "iss", "aud", "exp", "nbf", "iat", "jti",
        };

        var custom = principal.Claims
            .Where(c => !knownTypes.Contains(c.Type))
            .GroupBy(c => c.Type)
            .ToDictionary<IGrouping<string, System.Security.Claims.Claim>, string, object>(
                g => g.Key,
                g => g.Count() == 1 ? g.First().Value : g.Select(c => c.Value).ToList());

        return new Claims(sub, roles, permissions, groups, tenantId, custom);
    }

    private static IReadOnlyList<string> GetClaimValues(ClaimsPrincipal principal, string claimType)
    {
        var values = principal.FindAll(claimType).Select(c => c.Value).ToList();
        if (values.Count == 0)
        {
            return Array.Empty<string>();
        }

        // Handle JSON array values (e.g., roles: ["admin", "user"])
        if (values.Count == 1 && values[0].StartsWith('['))
        {
            try
            {
                var parsed = JsonSerializer.Deserialize<string[]>(values[0]);
                if (parsed is not null)
                {
                    return parsed;
                }
            }
            catch (JsonException)
            {
                // Not valid JSON array, return as-is
            }
        }

        return values;
    }
}
