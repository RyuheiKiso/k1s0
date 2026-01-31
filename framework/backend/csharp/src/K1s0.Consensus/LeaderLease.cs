namespace K1s0.Consensus;

/// <summary>
/// Represents a leader lease held by a node.
/// </summary>
/// <param name="LeaseKey">The key identifying the leadership group.</param>
/// <param name="HolderId">The unique identifier of the node holding the lease.</param>
/// <param name="FenceToken">Monotonically increasing token to prevent stale leaders from writing.</param>
/// <param name="ExpiresAt">When this lease expires if not renewed.</param>
public sealed record LeaderLease(
    string LeaseKey,
    string HolderId,
    ulong FenceToken,
    DateTimeOffset ExpiresAt)
{
    /// <summary>
    /// Returns <c>true</c> if this lease has not yet expired.
    /// </summary>
    public bool IsValid => DateTimeOffset.UtcNow < ExpiresAt;
}
