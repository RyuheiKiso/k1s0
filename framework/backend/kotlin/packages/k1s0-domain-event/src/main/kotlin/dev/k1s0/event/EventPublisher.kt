package dev.k1s0.event

/**
 * Interface for publishing domain events.
 */
public fun interface EventPublisher {
    /**
     * Publishes a domain event.
     *
     * @param envelope The event envelope to publish.
     */
    public suspend fun publish(envelope: EventEnvelope)
}

/**
 * Interface for subscribing to domain events.
 */
public fun interface EventSubscriber {
    /**
     * Handles a received domain event.
     *
     * @param envelope The event envelope received.
     */
    public suspend fun handle(envelope: EventEnvelope)
}
