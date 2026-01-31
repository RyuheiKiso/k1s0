using System.Collections.Concurrent;
using System.Security.Cryptography;
using System.Text;

namespace K1s0.Auth;

/// <summary>
/// Manages refresh token issuance, verification, and revocation using HMACSHA256 signatures.
/// </summary>
public class RefreshTokenManager : IRefreshTokenStore
{
    private readonly byte[] _signingKey;
    private readonly ConcurrentDictionary<string, TokenEntry> _tokens = new();
    private readonly TimeSpan _tokenLifetime;

    /// <summary>
    /// Initializes a new instance of the <see cref="RefreshTokenManager"/> class.
    /// </summary>
    /// <param name="signingKey">The HMAC signing key (minimum 32 bytes recommended).</param>
    /// <param name="tokenLifetime">The lifetime of issued refresh tokens. Defaults to 7 days.</param>
    public RefreshTokenManager(byte[] signingKey, TimeSpan? tokenLifetime = null)
    {
        _signingKey = signingKey ?? throw new ArgumentNullException(nameof(signingKey));
        _tokenLifetime = tokenLifetime ?? TimeSpan.FromDays(7);
    }

    /// <summary>
    /// Issues a new refresh token for the specified subject.
    /// </summary>
    /// <param name="sub">The subject identifier.</param>
    /// <returns>The issued refresh token string.</returns>
    public string Issue(string sub)
    {
        var payload = $"{sub}:{Guid.NewGuid():N}:{DateTime.UtcNow.Ticks}";
        var signature = Sign(payload);
        var token = $"{Convert.ToBase64String(Encoding.UTF8.GetBytes(payload))}.{Convert.ToBase64String(signature)}";
        var expiry = DateTime.UtcNow.Add(_tokenLifetime);

        _tokens[token] = new TokenEntry(sub, expiry);
        return token;
    }

    /// <summary>
    /// Verifies a refresh token and returns the associated subject.
    /// </summary>
    /// <param name="token">The refresh token to verify.</param>
    /// <returns>The subject identifier, or <c>null</c> if the token is invalid or expired.</returns>
    public string? Verify(string token)
    {
        var parts = token.Split('.');
        if (parts.Length != 2)
        {
            return null;
        }

        try
        {
            var payload = Encoding.UTF8.GetString(Convert.FromBase64String(parts[0]));
            var expectedSignature = Sign(payload);
            var actualSignature = Convert.FromBase64String(parts[1]);

            if (!CryptographicOperations.FixedTimeEquals(expectedSignature, actualSignature))
            {
                return null;
            }
        }
        catch (FormatException)
        {
            return null;
        }

        if (!_tokens.TryGetValue(token, out var entry))
        {
            return null;
        }

        if (DateTime.UtcNow >= entry.Expiry)
        {
            _tokens.TryRemove(token, out _);
            return null;
        }

        return entry.Sub;
    }

    /// <inheritdoc />
    public Task StoreAsync(string token, string sub, DateTime expiry, CancellationToken ct = default)
    {
        _tokens[token] = new TokenEntry(sub, expiry);
        return Task.CompletedTask;
    }

    /// <inheritdoc />
    public Task<string?> GetSubjectAsync(string token, CancellationToken ct = default)
    {
        return Task.FromResult(Verify(token));
    }

    /// <inheritdoc />
    public Task RevokeAsync(string token, CancellationToken ct = default)
    {
        _tokens.TryRemove(token, out _);
        return Task.CompletedTask;
    }

    private byte[] Sign(string payload)
    {
        using var hmac = new HMACSHA256(_signingKey);
        return hmac.ComputeHash(Encoding.UTF8.GetBytes(payload));
    }

    private sealed record TokenEntry(string Sub, DateTime Expiry);
}
