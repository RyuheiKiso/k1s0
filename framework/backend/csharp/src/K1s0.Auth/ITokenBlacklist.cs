namespace K1s0.Auth;

/// <summary>
/// Interface for managing a token blacklist (e.g., for logout/revocation).
/// </summary>
public interface ITokenBlacklist
{
    /// <summary>
    /// Checks whether the specified token identifier is blacklisted.
    /// </summary>
    /// <param name="jti">The JWT ID (jti) to check.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> if the token is blacklisted; otherwise <c>false</c>.</returns>
    Task<bool> IsBlacklistedAsync(string jti, CancellationToken ct = default);

    /// <summary>
    /// Adds a token identifier to the blacklist.
    /// </summary>
    /// <param name="jti">The JWT ID (jti) to blacklist.</param>
    /// <param name="expiry">When the blacklist entry should expire (typically matching the token's expiry).</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>A task representing the asynchronous operation.</returns>
    Task AddAsync(string jti, DateTime expiry, CancellationToken ct = default);
}
