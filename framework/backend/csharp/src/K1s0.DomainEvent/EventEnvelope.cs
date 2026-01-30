using System.Text.Json;

namespace K1s0.DomainEvent;

/// <summary>
/// Wraps a domain event with its metadata and serialized payload.
/// </summary>
/// <param name="EventType">The type identifier for the event.</param>
/// <param name="Metadata">The metadata associated with this event.</param>
/// <param name="Payload">The JSON-serialized event payload.</param>
public sealed record EventEnvelope(
    string EventType,
    EventMetadata Metadata,
    string Payload)
{
    /// <summary>
    /// Creates an <see cref="EventEnvelope"/> from a domain event, automatically generating metadata.
    /// </summary>
    /// <param name="domainEvent">The domain event to wrap.</param>
    /// <param name="source">The source service or component producing the event.</param>
    /// <param name="correlationId">Optional correlation identifier.</param>
    /// <param name="causationId">Optional causation identifier.</param>
    /// <returns>A new <see cref="EventEnvelope"/> containing the serialized event and metadata.</returns>
    public static EventEnvelope Wrap(
        IDomainEvent domainEvent,
        string source,
        string? correlationId = null,
        string? causationId = null)
    {
        ArgumentNullException.ThrowIfNull(domainEvent);
        ArgumentException.ThrowIfNullOrWhiteSpace(source);

        var metadata = new EventMetadata(
            Guid.NewGuid(),
            DateTimeOffset.UtcNow,
            source,
            correlationId,
            causationId);

        var payload = JsonSerializer.Serialize(domainEvent, domainEvent.GetType());
        return new EventEnvelope(domainEvent.EventType, metadata, payload);
    }
}
