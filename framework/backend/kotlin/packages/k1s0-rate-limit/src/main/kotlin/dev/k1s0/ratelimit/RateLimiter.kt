package dev.k1s0.ratelimit

import kotlin.time.Duration

/**
 * Rate limiter interface.
 *
 * Implementations control the rate at which requests are allowed through.
 */
public interface RateLimiter {
    /** Attempts to acquire a permit. Returns true if allowed. */
    public suspend fun tryAcquire(): Boolean

    /** Returns the estimated duration until the next token is available. */
    public fun timeUntilAvailable(): Duration

    /** Returns the number of currently available tokens. */
    public fun availableTokens(): Long

    /** Returns current rate limit statistics. */
    public fun stats(): RateLimitStats
}

/** Rate limit statistics snapshot. */
public data class RateLimitStats(
    val allowed: Long,
    val rejected: Long,
    val total: Long,
    val available: Long,
)
