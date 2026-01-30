namespace K1s0.Cache;

/// <summary>
/// Provides Redis list operations.
/// </summary>
public interface IListOperations
{
    /// <summary>
    /// Prepends a value to the head of a list.
    /// </summary>
    /// <param name="key">The list key.</param>
    /// <param name="value">The value to prepend.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The length of the list after the push.</returns>
    Task<long> LPushAsync(string key, string value, CancellationToken ct = default);

    /// <summary>
    /// Appends a value to the tail of a list.
    /// </summary>
    /// <param name="key">The list key.</param>
    /// <param name="value">The value to append.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The length of the list after the push.</returns>
    Task<long> RPushAsync(string key, string value, CancellationToken ct = default);

    /// <summary>
    /// Removes and returns the first element of a list.
    /// </summary>
    /// <param name="key">The list key.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The popped value, or <c>null</c> if the list is empty.</returns>
    Task<string?> LPopAsync(string key, CancellationToken ct = default);

    /// <summary>
    /// Removes and returns the last element of a list.
    /// </summary>
    /// <param name="key">The list key.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>The popped value, or <c>null</c> if the list is empty.</returns>
    Task<string?> RPopAsync(string key, CancellationToken ct = default);

    /// <summary>
    /// Returns a range of elements from a list.
    /// </summary>
    /// <param name="key">The list key.</param>
    /// <param name="start">The start index (inclusive).</param>
    /// <param name="stop">The stop index (inclusive). Use <c>-1</c> for the last element.</param>
    /// <param name="ct">A cancellation token.</param>
    /// <returns>A list of values in the specified range.</returns>
    Task<List<string>> LRangeAsync(string key, long start, long stop, CancellationToken ct = default);
}
