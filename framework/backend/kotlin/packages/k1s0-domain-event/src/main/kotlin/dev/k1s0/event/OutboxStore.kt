package dev.k1s0.event

import io.github.oshai.kotlinlogging.KotlinLogging

private val logger = KotlinLogging.logger {}

/**
 * Outbox pattern interface for reliable domain event publishing.
 *
 * Events are first stored in a local outbox table within the same
 * database transaction as the business operation, then published
 * asynchronously by a separate relay process.
 */
public interface OutboxStore {

    /**
     * Stores an event in the outbox for later publishing.
     *
     * @param envelope The event to store.
     */
    public suspend fun store(envelope: EventEnvelope)

    /**
     * Retrieves unpublished events from the outbox.
     *
     * @param limit Maximum number of events to retrieve.
     * @return List of unpublished event envelopes.
     */
    public suspend fun fetchUnpublished(limit: Int = 100): List<EventEnvelope>

    /**
     * Marks an event as published.
     *
     * @param eventId The ID of the event to mark.
     */
    public suspend fun markPublished(eventId: String)
}

/**
 * In-memory outbox store for testing purposes.
 */
public class InMemoryOutboxStore : OutboxStore {

    private val events = mutableListOf<Pair<EventEnvelope, Boolean>>()

    override suspend fun store(envelope: EventEnvelope) {
        events.add(envelope to false)
        logger.debug { "Stored event in outbox: ${envelope.eventId}" }
    }

    override suspend fun fetchUnpublished(limit: Int): List<EventEnvelope> =
        events.filter { !it.second }.take(limit).map { it.first }

    override suspend fun markPublished(eventId: String) {
        val index = events.indexOfFirst { it.first.eventId == eventId }
        if (index >= 0) {
            events[index] = events[index].first to true
        }
    }
}
