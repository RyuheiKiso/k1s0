package dev.k1s0.android.realtime

import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class WebSocketClientTest {

    @Test
    fun `ReconnectionStrategy returns correct delay for first attempt`() {
        val strategy = ReconnectionStrategy(
            initialDelayMs = 1000,
            maxDelayMs = 30000,
            jitterFactor = 0.0,
        )

        val delay = strategy.delayForAttempt(0)
        assertEquals(1000L, delay)
    }

    @Test
    fun `ReconnectionStrategy applies exponential backoff`() {
        val strategy = ReconnectionStrategy(
            initialDelayMs = 1000,
            maxDelayMs = 30000,
            backoffMultiplier = 2.0,
            jitterFactor = 0.0,
        )

        assertEquals(1000L, strategy.delayForAttempt(0))
        assertEquals(2000L, strategy.delayForAttempt(1))
        assertEquals(4000L, strategy.delayForAttempt(2))
        assertEquals(8000L, strategy.delayForAttempt(3))
    }

    @Test
    fun `ReconnectionStrategy caps delay at maxDelayMs`() {
        val strategy = ReconnectionStrategy(
            initialDelayMs = 1000,
            maxDelayMs = 5000,
            backoffMultiplier = 2.0,
            jitterFactor = 0.0,
        )

        assertEquals(5000L, strategy.delayForAttempt(10))
    }

    @Test
    fun `ReconnectionStrategy returns null when max attempts exceeded`() {
        val strategy = ReconnectionStrategy(maxAttempts = 3)

        assertNotNull(strategy.delayForAttempt(0))
        assertNotNull(strategy.delayForAttempt(2))
        assertNull(strategy.delayForAttempt(3))
    }

    @Test
    fun `ConnectionState enum has all expected values`() {
        val states = ConnectionState.entries
        assertEquals(4, states.size)
        assertTrue(states.contains(ConnectionState.DISCONNECTED))
        assertTrue(states.contains(ConnectionState.CONNECTING))
        assertTrue(states.contains(ConnectionState.CONNECTED))
        assertTrue(states.contains(ConnectionState.RECONNECTING))
    }

    @Test
    fun `OfflineQueue enqueues and drains messages`() = runTest {
        val queue = OfflineQueue()
        queue.enqueue("msg1")
        queue.enqueue("msg2")

        assertEquals(2, queue.size())

        val messages = queue.drain()
        assertEquals(listOf("msg1", "msg2"), messages)
        assertTrue(queue.isEmpty())
    }

    @Test
    fun `OfflineQueue drops oldest when at capacity`() = runTest {
        val queue = OfflineQueue(maxSize = 2)
        queue.enqueue("msg1")
        queue.enqueue("msg2")
        queue.enqueue("msg3")

        val messages = queue.drain()
        assertEquals(listOf("msg2", "msg3"), messages)
    }

    @Test
    fun `OfflineQueue clear empties the queue`() = runTest {
        val queue = OfflineQueue()
        queue.enqueue("msg1")
        queue.clear()
        assertTrue(queue.isEmpty())
    }

    @Test
    fun `SseEvent holds event data`() {
        val event = SseEvent(
            id = "1",
            event = "update",
            data = "payload",
        )
        assertEquals("1", event.id)
        assertEquals("update", event.event)
        assertEquals("payload", event.data)
    }
}
