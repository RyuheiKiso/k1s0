namespace K1s0.Cache;

/// <summary>
/// Provides Redis set operations.
/// </summary>
public interface ISetOperations
{
    /// <summary>
    /// Adds a member to a set.
    /// </summary>
    /// <param name="key">The set key.</param>
    /// <param name="value">The value to add.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns><c>true</c> if the value was added; <c>false</c> if it already existed.</returns>
    Task<bool> SAddAsync(string key, string value, CancellationToken ct = default);

    /// <summary>
    /// Removes a member from a set.
    /// </summary>
    /// <param name="key">The set key.</param>
    /// <param name="value">The value to remove.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns><c>true</c> if the value was removed; <c>false</c> if it did not exist.</returns>
    Task<bool> SRemoveAsync(string key, string value, CancellationToken ct = default);

    /// <summary>
    /// Returns all members of a set.
    /// </summary>
    /// <param name="key">The set key.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>A set of all member values.</returns>
    Task<HashSet<string>> SMembersAsync(string key, CancellationToken ct = default);

    /// <summary>
    /// Checks whether a value is a member of a set.
    /// </summary>
    /// <param name="key">The set key.</param>
    /// <param name="value">The value to check.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns><c>true</c> if the value is a member; otherwise, <c>false</c>.</returns>
    Task<bool> SIsMemberAsync(string key, string value, CancellationToken ct = default);
}
