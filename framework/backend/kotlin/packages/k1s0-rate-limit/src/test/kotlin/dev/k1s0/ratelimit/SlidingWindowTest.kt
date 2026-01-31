package dev.k1s0.ratelimit

import io.kotest.matchers.shouldBe
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Test
import kotlin.time.Duration.Companion.seconds

class SlidingWindowTest {

    @Test
    fun `tryAcquire succeeds within limit`() = runTest {
        val window = SlidingWindow(SlidingWindowConfig(maxRequests = 5, window = 60.seconds))

        window.tryAcquire() shouldBe true
        window.tryAcquire() shouldBe true
        window.stats().allowed shouldBe 2
    }

    @Test
    fun `tryAcquire rejects when limit exceeded`() = runTest {
        val window = SlidingWindow(SlidingWindowConfig(maxRequests = 2, window = 60.seconds))

        window.tryAcquire() shouldBe true
        window.tryAcquire() shouldBe true
        window.tryAcquire() shouldBe false

        val stats = window.stats()
        stats.allowed shouldBe 2
        stats.rejected shouldBe 1
    }

    @Test
    fun `availableTokens returns correct count`() = runTest {
        val window = SlidingWindow(SlidingWindowConfig(maxRequests = 3, window = 60.seconds))

        window.availableTokens() shouldBe 3
        window.tryAcquire()
        window.availableTokens() shouldBe 2
    }

    @Test
    fun `stats tracks totals correctly`() = runTest {
        val window = SlidingWindow(SlidingWindowConfig(maxRequests = 1, window = 60.seconds))

        window.tryAcquire()
        window.tryAcquire()

        val stats = window.stats()
        stats.total shouldBe 2
        stats.allowed shouldBe 1
        stats.rejected shouldBe 1
    }
}
