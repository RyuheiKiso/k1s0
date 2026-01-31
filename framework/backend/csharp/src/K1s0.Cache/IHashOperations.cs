namespace K1s0.Cache;

/// <summary>
/// Provides Redis hash operations.
/// </summary>
public interface IHashOperations
{
    /// <summary>
    /// Gets the value of a field in a hash.
    /// </summary>
    /// <param name="key">The hash key.</param>
    /// <param name="field">The field name.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The field value, or <c>null</c> if the field does not exist.</returns>
    Task<string?> HGetAsync(string key, string field, CancellationToken ct = default);

    /// <summary>
    /// Sets the value of a field in a hash.
    /// </summary>
    /// <param name="key">The hash key.</param>
    /// <param name="field">The field name.</param>
    /// <param name="value">The value to set.</param>
    /// <param name="ct">A cancellation token.</param>
    Task HSetAsync(string key, string field, string value, CancellationToken ct = default);

    /// <summary>
    /// Deletes a field from a hash.
    /// </summary>
    /// <param name="key">The hash key.</param>
    /// <param name="field">The field name.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns><c>true</c> if the field was removed; otherwise, <c>false</c>.</returns>
    Task<bool> HDeleteAsync(string key, string field, CancellationToken ct = default);

    /// <summary>
    /// Gets all fields and values of a hash.
    /// </summary>
    /// <param name="key">The hash key.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>A dictionary of field-value pairs.</returns>
    Task<Dictionary<string, string>> HGetAllAsync(string key, CancellationToken ct = default);
}
