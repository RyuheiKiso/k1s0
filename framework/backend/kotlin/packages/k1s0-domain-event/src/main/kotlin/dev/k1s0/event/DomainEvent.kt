package dev.k1s0.event

import kotlinx.serialization.Serializable
import java.time.Instant
import java.util.UUID

/**
 * Base interface for all domain events in k1s0 services.
 */
public interface DomainEvent {
    /** Unique identifier for this event occurrence. */
    public val eventId: String

    /** The type name of the event (e.g., "order.created"). */
    public val eventType: String

    /** When the event occurred. */
    public val occurredAt: String
}

/**
 * Envelope wrapping a domain event with metadata for transport.
 *
 * @property eventId Unique event identifier.
 * @property eventType The event type discriminator.
 * @property occurredAt ISO-8601 timestamp of when the event occurred.
 * @property payload The serialized event payload as a JSON string.
 * @property aggregateId The aggregate that produced this event.
 */
@Serializable
public data class EventEnvelope(
    val eventId: String = UUID.randomUUID().toString(),
    val eventType: String,
    val occurredAt: String = Instant.now().toString(),
    val payload: String,
    val aggregateId: String,
)
