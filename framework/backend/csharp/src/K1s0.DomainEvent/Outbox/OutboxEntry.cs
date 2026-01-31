namespace K1s0.DomainEvent.Outbox;

/// <summary>
/// Represents a persisted outbox entry for reliable event delivery.
/// </summary>
/// <param name="Id">Unique identifier for the outbox entry.</param>
/// <param name="EventType">The type identifier of the event.</param>
/// <param name="Payload">The JSON-serialized event envelope.</param>
/// <param name="Status">The current processing status.</param>
/// <param name="RetryCount">The number of publish attempts made.</param>
/// <param name="CreatedAt">The timestamp when the entry was created.</param>
/// <param name="UpdatedAt">The timestamp when the entry was last updated.</param>
public sealed record OutboxEntry(
    Guid Id,
    string EventType,
    string Payload,
    OutboxStatus Status,
    int RetryCount,
    DateTimeOffset CreatedAt,
    DateTimeOffset UpdatedAt);
