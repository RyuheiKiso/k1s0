package dev.k1s0.ratelimit

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeSource

private val logger = KotlinLogging.logger {}

/**
 * Sliding window rate limiter configuration.
 *
 * @property maxRequests Maximum number of requests allowed in the window.
 * @property window Duration of the sliding window.
 */
public data class SlidingWindowConfig(
    val maxRequests: Long = 1000,
    val window: Duration = 60.seconds,
)

/**
 * Sliding window rate limiter.
 *
 * Tracks request timestamps within a rolling time window and rejects
 * requests when the limit is exceeded.
 */
public class SlidingWindow(
    config: SlidingWindowConfig = SlidingWindowConfig(),
) : RateLimiter {
    private val maxRequests = config.maxRequests
    private val window = config.window
    private val timestamps = ArrayDeque<TimeSource.Monotonic.ValueTimeMark>()
    private val mutex = Mutex()
    private var allowed = 0L
    private var rejected = 0L

    private fun evictExpired() {
        val now = TimeSource.Monotonic.markNow()
        while (timestamps.isNotEmpty()) {
            val oldest = timestamps.first()
            if (oldest.elapsedNow() >= window) {
                timestamps.removeFirst()
            } else {
                break
            }
        }
    }

    override suspend fun tryAcquire(): Boolean = mutex.withLock {
        evictExpired()
        if (timestamps.size < maxRequests) {
            timestamps.addLast(TimeSource.Monotonic.markNow())
            allowed++
            logger.trace { "Request allowed (count=${timestamps.size}/$maxRequests)" }
            true
        } else {
            rejected++
            logger.debug { "Request rejected (count=${timestamps.size}/$maxRequests)" }
            false
        }
    }

    override fun timeUntilAvailable(): Duration {
        if (timestamps.isEmpty() || timestamps.size < maxRequests) {
            return Duration.ZERO
        }
        val oldest = timestamps.first()
        val elapsed = oldest.elapsedNow()
        return if (elapsed >= window) Duration.ZERO else window - elapsed
    }

    override fun availableTokens(): Long {
        evictExpired()
        return maxOf(0L, maxRequests - timestamps.size)
    }

    override fun stats(): RateLimitStats = RateLimitStats(
        allowed = allowed,
        rejected = rejected,
        total = allowed + rejected,
        available = availableTokens(),
    )
}
