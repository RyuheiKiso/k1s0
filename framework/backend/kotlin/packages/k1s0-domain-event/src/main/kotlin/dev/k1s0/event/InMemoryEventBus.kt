package dev.k1s0.event

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

private val logger = KotlinLogging.logger {}

/**
 * In-memory event bus for local publish/subscribe within a single process.
 *
 * Suitable for testing and single-instance deployments.
 * For distributed systems, use an outbox-based implementation.
 */
public class InMemoryEventBus : EventPublisher {

    private val mutex = Mutex()
    private val subscribers = mutableMapOf<String, MutableList<EventSubscriber>>()
    private val publishedEvents = mutableListOf<EventEnvelope>()

    /**
     * Registers a subscriber for a specific event type.
     *
     * @param eventType The event type to subscribe to.
     * @param subscriber The subscriber to notify.
     */
    public suspend fun subscribe(eventType: String, subscriber: EventSubscriber) {
        mutex.withLock {
            subscribers.getOrPut(eventType) { mutableListOf() }.add(subscriber)
        }
        logger.debug { "Subscriber registered for event type: $eventType" }
    }

    override suspend fun publish(envelope: EventEnvelope) {
        val handlers = mutex.withLock {
            publishedEvents.add(envelope)
            subscribers[envelope.eventType]?.toList() ?: emptyList()
        }
        handlers.forEach { it.handle(envelope) }
        logger.debug { "Published event: ${envelope.eventType} (${envelope.eventId})" }
    }

    /** Returns all published events (for testing). */
    public suspend fun getPublishedEvents(): List<EventEnvelope> = mutex.withLock {
        publishedEvents.toList()
    }
}
