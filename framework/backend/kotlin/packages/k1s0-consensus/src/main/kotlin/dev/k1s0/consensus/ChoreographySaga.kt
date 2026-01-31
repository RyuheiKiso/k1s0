package dev.k1s0.consensus

import dev.k1s0.event.EventEnvelope
import dev.k1s0.event.EventPublisher
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withTimeout
import java.util.UUID

private val logger = KotlinLogging.logger {}

/**
 * Handler for a single step in a choreography-based saga.
 *
 * Each handler listens for a specific event type and produces the next
 * event or compensating event.
 */
public interface EventStepHandler {
    /** The event type this handler responds to. */
    public val eventType: String

    /**
     * Handles the event and returns the next event to publish.
     *
     * @param envelope The received event.
     * @param context Mutable saga context.
     * @return The next event envelope to publish, or null if this is the final step.
     */
    public suspend fun handle(envelope: EventEnvelope, context: MutableMap<String, Any>): EventEnvelope?

    /**
     * Produces a compensating event when this step needs to be rolled back.
     *
     * @param envelope The original event that triggered this step.
     * @param context The saga context.
     * @return The compensation event to publish.
     */
    public suspend fun compensate(envelope: EventEnvelope, context: MutableMap<String, Any>): EventEnvelope?
}

/**
 * Builder for choreography-based saga definitions.
 *
 * Usage:
 * ```kotlin
 * val saga = choreography("order-flow") {
 *     on("order.created") { envelope, ctx ->
 *         // Process and return next event
 *         EventEnvelope(eventType = "inventory.reserved", payload = "...", aggregateId = "...")
 *     } compensateWith { envelope, ctx ->
 *         EventEnvelope(eventType = "order.cancelled", payload = "...", aggregateId = "...")
 *     }
 *     timeout(30_000L)
 * }
 * ```
 */
public class ChoreographySagaBuilder(private val name: String) {

    private val handlers = mutableListOf<EventStepHandler>()
    private var timeoutMs: Long = 30_000L

    /**
     * Registers an event handler step.
     */
    public fun on(
        eventType: String,
        handler: suspend (EventEnvelope, MutableMap<String, Any>) -> EventEnvelope?,
    ): CompensationBuilder {
        val step = DslEventStepHandler(eventType, handler)
        handlers.add(step)
        return CompensationBuilder(step)
    }

    /**
     * Sets the timeout for the entire choreography saga.
     */
    public fun timeout(ms: Long) {
        timeoutMs = ms
    }

    public fun build(): ChoreographySagaDefinition = ChoreographySagaDefinition(
        name = name,
        handlers = handlers.toList(),
        timeoutMs = timeoutMs,
    )

    public class CompensationBuilder(private val step: DslEventStepHandler) {
        public infix fun compensateWith(
            compensator: suspend (EventEnvelope, MutableMap<String, Any>) -> EventEnvelope?,
        ) {
            step.compensator = compensator
        }
    }

    internal class DslEventStepHandler(
        override val eventType: String,
        private val handler: suspend (EventEnvelope, MutableMap<String, Any>) -> EventEnvelope?,
        internal var compensator: (suspend (EventEnvelope, MutableMap<String, Any>) -> EventEnvelope?)? = null,
    ) : EventStepHandler {

        override suspend fun handle(envelope: EventEnvelope, context: MutableMap<String, Any>): EventEnvelope? =
            handler(envelope, context)

        override suspend fun compensate(envelope: EventEnvelope, context: MutableMap<String, Any>): EventEnvelope? =
            compensator?.invoke(envelope, context)
    }
}

/**
 * Definition of a choreography-based saga.
 *
 * @property name The name of the saga.
 * @property handlers Ordered list of event step handlers.
 * @property timeoutMs Timeout for the entire saga in milliseconds.
 */
public data class ChoreographySagaDefinition(
    val name: String,
    val handlers: List<EventStepHandler>,
    val timeoutMs: Long = 30_000L,
)

/**
 * Executes a choreography-based saga with timeout.
 *
 * @property publisher The event publisher for emitting saga events.
 * @property metrics Optional saga metrics.
 */
public class ChoreographySagaExecutor(
    private val publisher: EventPublisher,
    private val metrics: SagaMetrics? = null,
) {
    /**
     * Executes a choreography saga by processing a trigger event through
     * the handler chain with a timeout.
     *
     * @param definition The choreography saga definition.
     * @param triggerEvent The event that starts the saga.
     * @return The saga result.
     */
    public suspend fun execute(
        definition: ChoreographySagaDefinition,
        triggerEvent: EventEnvelope,
    ): SagaResult {
        val sagaId = UUID.randomUUID().toString()
        val context = mutableMapOf<String, Any>()
        val completedSteps = mutableListOf<Pair<EventStepHandler, EventEnvelope>>()

        logger.info { "Starting choreography saga '${definition.name}' (id=$sagaId)" }
        metrics?.sagaStarted(definition.name)

        return try {
            coroutineScope {
                withTimeout(definition.timeoutMs) {
                    var currentEvent: EventEnvelope? = triggerEvent

                    for (handler in definition.handlers) {
                        if (currentEvent == null) break
                        if (handler.eventType != currentEvent.eventType) continue

                        logger.debug { "Processing event '${handler.eventType}' in choreography '$sagaId'" }
                        val nextEvent = handler.handle(currentEvent, context)
                        completedSteps.add(handler to currentEvent)

                        if (nextEvent != null) {
                            publisher.publish(nextEvent)
                        }
                        currentEvent = nextEvent
                    }
                }
            }

            metrics?.sagaCompleted(definition.name)
            SagaResult(
                sagaId = sagaId,
                status = SagaStatus.COMPLETED,
                context = context,
                completedSteps = completedSteps.map { it.first.eventType },
            )
        } catch (e: Exception) {
            logger.error(e) { "Choreography saga '$sagaId' failed" }

            // Compensate in reverse
            val compensated = mutableListOf<String>()
            for ((handler, event) in completedSteps.reversed()) {
                try {
                    val compEvent = handler.compensate(event, context)
                    if (compEvent != null) {
                        publisher.publish(compEvent)
                    }
                    compensated.add(handler.eventType)
                } catch (ce: Exception) {
                    logger.error(ce) { "Compensation failed for '${handler.eventType}' in choreography '$sagaId'" }
                }
            }

            SagaResult(
                sagaId = sagaId,
                status = SagaStatus.COMPENSATED,
                context = context,
                error = e.message,
                completedSteps = completedSteps.map { it.first.eventType },
                compensatedSteps = compensated,
            )
        }
    }
}

/**
 * Top-level DSL function for building a choreography saga definition.
 */
public fun choreography(name: String, block: ChoreographySagaBuilder.() -> Unit): ChoreographySagaDefinition {
    val builder = ChoreographySagaBuilder(name)
    builder.block()
    return builder.build()
}
