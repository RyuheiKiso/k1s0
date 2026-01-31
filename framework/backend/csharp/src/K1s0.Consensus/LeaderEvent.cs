namespace K1s0.Consensus;

/// <summary>
/// Types of leader election events.
/// </summary>
public enum LeaderEventType
{
    /// <summary>
    /// This node has become the leader.
    /// </summary>
    Elected,

    /// <summary>
    /// The leader lease was successfully renewed.
    /// </summary>
    Renewed,

    /// <summary>
    /// This node has lost leadership (lease expired or explicitly released).
    /// </summary>
    Lost,

    /// <summary>
    /// A different node has become the leader.
    /// </summary>
    Changed
}

/// <summary>
/// An event emitted by the leader election system.
/// </summary>
/// <param name="EventType">The type of leader event.</param>
/// <param name="Lease">The current lease state, if available.</param>
/// <param name="Timestamp">When this event occurred.</param>
public sealed record LeaderEvent(
    LeaderEventType EventType,
    LeaderLease? Lease,
    DateTimeOffset Timestamp)
{
    /// <summary>
    /// Creates a leader event with the current UTC timestamp.
    /// </summary>
    /// <param name="eventType">The event type.</param>
    /// <param name="lease">The associated lease.</param>
    /// <returns>A new <see cref="LeaderEvent"/>.</returns>
    public static LeaderEvent Create(LeaderEventType eventType, LeaderLease? lease = null) =>
        new(eventType, lease, DateTimeOffset.UtcNow);
}
