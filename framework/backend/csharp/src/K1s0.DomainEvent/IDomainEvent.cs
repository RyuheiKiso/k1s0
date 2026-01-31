namespace K1s0.DomainEvent;

/// <summary>
/// Represents a domain event that occurred within an aggregate.
/// </summary>
public interface IDomainEvent
{
    /// <summary>
    /// Gets the type identifier for this event (e.g., "order.created").
    /// </summary>
    string EventType { get; }

    /// <summary>
    /// Gets the identifier of the aggregate that produced this event.
    /// </summary>
    string AggregateId { get; }

    /// <summary>
    /// Gets the type of the aggregate that produced this event (e.g., "Order").
    /// </summary>
    string AggregateType { get; }
}
