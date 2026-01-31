package dev.k1s0.resilience

import io.kotest.assertions.throwables.shouldThrow
import io.kotest.matchers.shouldBe
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Test

class CircuitBreakerTest {

    @Test
    fun `circuit breaker starts closed`() = runTest {
        val cb = CircuitBreaker("test", failureThreshold = 2)

        cb.currentState() shouldBe CircuitBreaker.State.CLOSED
    }

    @Test
    fun `circuit breaker opens after failure threshold`() = runTest {
        val cb = CircuitBreaker("test", failureThreshold = 2)

        repeat(2) {
            try {
                cb.execute<Unit> { throw RuntimeException("fail") }
            } catch (_: RuntimeException) { /* expected */ }
        }

        cb.currentState() shouldBe CircuitBreaker.State.OPEN
    }

    @Test
    fun `open circuit breaker rejects calls`() = runTest {
        val cb = CircuitBreaker("test", failureThreshold = 1)

        try {
            cb.execute<Unit> { throw RuntimeException("fail") }
        } catch (_: RuntimeException) { /* expected */ }

        shouldThrow<CircuitBreakerOpenException> {
            cb.execute { "should not run" }
        }
    }

    @Test
    fun `successful call keeps circuit closed`() = runTest {
        val cb = CircuitBreaker("test", failureThreshold = 3)

        val result = cb.execute { "ok" }

        result shouldBe "ok"
        cb.currentState() shouldBe CircuitBreaker.State.CLOSED
    }
}
