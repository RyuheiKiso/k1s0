namespace K1s0.DomainEvent.Outbox;

/// <summary>
/// Represents the processing status of an outbox entry.
/// </summary>
public enum OutboxStatus
{
    /// <summary>The entry is pending publication.</summary>
    Pending,

    /// <summary>The entry has been successfully published.</summary>
    Published,

    /// <summary>The entry failed to publish after exhausting retries.</summary>
    Failed,
}
