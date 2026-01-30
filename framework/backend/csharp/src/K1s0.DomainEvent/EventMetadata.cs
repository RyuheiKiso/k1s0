namespace K1s0.DomainEvent;

/// <summary>
/// Metadata associated with a domain event.
/// </summary>
/// <param name="EventId">Unique identifier for this event instance.</param>
/// <param name="OccurredAt">The timestamp when the event occurred.</param>
/// <param name="Source">The service or component that produced the event.</param>
/// <param name="CorrelationId">Optional correlation identifier for tracing related events.</param>
/// <param name="CausationId">Optional identifier of the event or command that caused this event.</param>
public sealed record EventMetadata(
    Guid EventId,
    DateTimeOffset OccurredAt,
    string Source,
    string? CorrelationId = null,
    string? CausationId = null);
