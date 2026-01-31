package dev.k1s0.event

import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Test

class InMemoryEventBusTest {

    @Test
    fun `publish delivers event to subscribers`() = runTest {
        val bus = InMemoryEventBus()
        val received = mutableListOf<EventEnvelope>()

        bus.subscribe("order.created") { received.add(it) }

        val envelope = EventEnvelope(
            eventType = "order.created",
            payload = """{"orderId": "123"}""",
            aggregateId = "order-123",
        )
        bus.publish(envelope)

        received shouldHaveSize 1
        received[0].aggregateId shouldBe "order-123"
    }

    @Test
    fun `publish stores events for retrieval`() = runTest {
        val bus = InMemoryEventBus()

        bus.publish(
            EventEnvelope(
                eventType = "test.event",
                payload = "{}",
                aggregateId = "agg-1",
            ),
        )

        bus.getPublishedEvents() shouldHaveSize 1
    }
}
