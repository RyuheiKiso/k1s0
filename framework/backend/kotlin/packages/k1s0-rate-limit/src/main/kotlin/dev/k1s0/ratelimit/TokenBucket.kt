package dev.k1s0.ratelimit

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeSource

private val logger = KotlinLogging.logger {}

/**
 * Token bucket rate limiter configuration.
 *
 * @property capacity Maximum number of tokens in the bucket.
 * @property refillRate Tokens added per second.
 */
public data class TokenBucketConfig(
    val capacity: Long = 1000,
    val refillRate: Double = 100.0,
)

/**
 * Token bucket rate limiter.
 *
 * Tokens are continuously replenished at [TokenBucketConfig.refillRate] per second,
 * up to the [TokenBucketConfig.capacity] limit.
 */
public class TokenBucket(
    config: TokenBucketConfig = TokenBucketConfig(),
) : RateLimiter {
    private val capacity = config.capacity
    private val refillRate = config.refillRate
    private var tokens = config.capacity.toDouble()
    private var lastRefill = TimeSource.Monotonic.markNow()
    private val mutex = Mutex()
    private var allowed = 0L
    private var rejected = 0L

    private fun refill() {
        val elapsed = lastRefill.elapsedNow()
        tokens = minOf(capacity.toDouble(), tokens + elapsed.inWholeMilliseconds / 1000.0 * refillRate)
        lastRefill = TimeSource.Monotonic.markNow()
    }

    override suspend fun tryAcquire(): Boolean = mutex.withLock {
        refill()
        if (tokens >= 1.0) {
            tokens -= 1.0
            allowed++
            logger.trace { "Token acquired (available=${tokens.toLong()})" }
            true
        } else {
            rejected++
            logger.debug { "Token rejected (available=${tokens.toLong()})" }
            false
        }
    }

    override fun timeUntilAvailable(): Duration {
        val deficit = 1.0 - tokens
        return if (deficit <= 0.0) {
            Duration.ZERO
        } else {
            (deficit / refillRate).seconds
        }
    }

    override fun availableTokens(): Long = tokens.toLong()

    override fun stats(): RateLimitStats = RateLimitStats(
        allowed = allowed,
        rejected = rejected,
        total = allowed + rejected,
        available = tokens.toLong(),
    )
}
