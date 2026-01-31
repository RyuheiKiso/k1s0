package dev.k1s0.ratelimit

import io.kotest.matchers.shouldBe
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Test
import kotlin.time.Duration

class TokenBucketTest {

    @Test
    fun `tryAcquire succeeds when tokens available`() = runTest {
        val bucket = TokenBucket(TokenBucketConfig(capacity = 10, refillRate = 10.0))

        bucket.tryAcquire() shouldBe true
        bucket.stats().allowed shouldBe 1
    }

    @Test
    fun `tryAcquire rejects when tokens exhausted`() = runTest {
        val bucket = TokenBucket(TokenBucketConfig(capacity = 2, refillRate = 0.0))

        bucket.tryAcquire() shouldBe true
        bucket.tryAcquire() shouldBe true
        bucket.tryAcquire() shouldBe false

        val stats = bucket.stats()
        stats.allowed shouldBe 2
        stats.rejected shouldBe 1
    }

    @Test
    fun `availableTokens returns correct count`() = runTest {
        val bucket = TokenBucket(TokenBucketConfig(capacity = 5, refillRate = 0.0))

        bucket.availableTokens() shouldBe 5
        bucket.tryAcquire()
        bucket.availableTokens() shouldBe 4
    }

    @Test
    fun `timeUntilAvailable returns zero when tokens available`() = runTest {
        val bucket = TokenBucket(TokenBucketConfig(capacity = 5, refillRate = 10.0))

        bucket.timeUntilAvailable() shouldBe Duration.ZERO
    }

    @Test
    fun `stats tracks totals correctly`() = runTest {
        val bucket = TokenBucket(TokenBucketConfig(capacity = 1, refillRate = 0.0))

        bucket.tryAcquire()
        bucket.tryAcquire()

        val stats = bucket.stats()
        stats.total shouldBe 2
        stats.allowed shouldBe 1
        stats.rejected shouldBe 1
    }
}
