namespace K1s0.Auth;

/// <summary>
/// Interface for storing and retrieving refresh tokens.
/// </summary>
public interface IRefreshTokenStore
{
    /// <summary>
    /// Stores a refresh token associated with a subject.
    /// </summary>
    /// <param name="token">The refresh token.</param>
    /// <param name="sub">The subject identifier.</param>
    /// <param name="expiry">When the token expires.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>A task representing the asynchronous operation.</returns>
    Task StoreAsync(string token, string sub, DateTime expiry, CancellationToken ct = default);

    /// <summary>
    /// Gets the subject associated with a refresh token.
    /// </summary>
    /// <param name="token">The refresh token.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The subject identifier, or <c>null</c> if the token is not found or expired.</returns>
    Task<string?> GetSubjectAsync(string token, CancellationToken ct = default);

    /// <summary>
    /// Revokes a refresh token.
    /// </summary>
    /// <param name="token">The refresh token to revoke.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>A task representing the asynchronous operation.</returns>
    Task RevokeAsync(string token, CancellationToken ct = default);
}
