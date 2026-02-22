using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using Microsoft.IdentityModel.Tokens;

namespace K1s0.System.Auth;

public sealed class JwksVerifier : IJwksVerifier
{
    private readonly IJwksFetcher _jwksFetcher;
    private readonly AuthConfig _config;
    private readonly SemaphoreSlim _cacheLock = new(1, 1);
    private IReadOnlyList<JsonWebKey>? _cachedKeys;
    private DateTime _cacheExpiry = DateTime.MinValue;

    public JwksVerifier(IJwksFetcher jwksFetcher, AuthConfig config)
    {
        _jwksFetcher = jwksFetcher;
        _config = config;
    }

    public async Task<TokenClaims> VerifyTokenAsync(string token, CancellationToken ct = default)
    {
        var keys = await GetKeysAsync(ct).ConfigureAwait(false);

        var validationParameters = new TokenValidationParameters
        {
            ValidIssuer = _config.Issuer,
            ValidAudience = _config.Audience,
            IssuerSigningKeys = keys,
            ValidateIssuer = true,
            ValidateAudience = true,
            ValidateLifetime = true,
            ValidateIssuerSigningKey = true,
            ClockSkew = TimeSpan.FromSeconds(30),
        };

        var handler = new JwtSecurityTokenHandler();

        ClaimsPrincipal principal;
        SecurityToken validatedToken;
        try
        {
            principal = handler.ValidateToken(token, validationParameters, out validatedToken);
        }
        catch (SecurityTokenExpiredException ex)
        {
            throw new AuthException("TOKEN_EXPIRED", "Token has expired", ex);
        }
        catch (SecurityTokenException ex)
        {
            throw new AuthException("INVALID_TOKEN", $"Token validation failed: {ex.Message}", ex);
        }
        catch (ArgumentException ex)
        {
            throw new AuthException("INVALID_TOKEN", $"Token validation failed: {ex.Message}", ex);
        }

        return ExtractClaims(principal, (JwtSecurityToken)validatedToken);
    }

    private async Task<IReadOnlyList<JsonWebKey>> GetKeysAsync(CancellationToken ct)
    {
        // Fast path: check cache without lock
        if (_cachedKeys is not null && DateTime.UtcNow < _cacheExpiry)
        {
            return _cachedKeys;
        }

        await _cacheLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            // Double-check after acquiring lock
            if (_cachedKeys is not null && DateTime.UtcNow < _cacheExpiry)
            {
                return _cachedKeys;
            }

            _cachedKeys = await _jwksFetcher.FetchKeysAsync(_config.JwksUrl, ct).ConfigureAwait(false);
            _cacheExpiry = DateTime.UtcNow.AddSeconds(_config.CacheTtlSeconds);
            return _cachedKeys;
        }
        finally
        {
            _cacheLock.Release();
        }
    }

    private static TokenClaims ExtractClaims(ClaimsPrincipal principal, JwtSecurityToken jwt)
    {
        var roles = principal.FindAll(ClaimTypes.Role)
            .Select(c => c.Value)
            .Concat(principal.FindAll("realm_access/roles").Select(c => c.Value))
            .Distinct()
            .ToList();

        var resourceAccess = new Dictionary<string, IReadOnlyList<string>>();
        foreach (var claim in principal.FindAll("resource_access"))
        {
            var parts = claim.Value.Split(':');
            if (parts.Length == 2)
            {
                var resource = parts[0];
                var role = parts[1];
                if (!resourceAccess.TryGetValue(resource, out var existingRoles))
                {
                    existingRoles = new List<string>();
                    resourceAccess[resource] = existingRoles;
                }

                ((List<string>)existingRoles).Add(role);
            }
        }

        return new TokenClaims
        {
            Sub = jwt.Subject ?? string.Empty,
            Iss = jwt.Issuer ?? string.Empty,
            Aud = jwt.Audiences.FirstOrDefault() ?? string.Empty,
            Scope = principal.FindFirst("scope")?.Value,
            Exp = new DateTimeOffset(jwt.ValidTo).ToUnixTimeSeconds(),
            Iat = new DateTimeOffset(jwt.IssuedAt).ToUnixTimeSeconds(),
            Roles = roles.AsReadOnly(),
            ResourceAccess = resourceAccess,
        };
    }
}
