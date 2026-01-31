package dev.k1s0.android.realtime

import kotlin.math.min
import kotlin.math.pow
import kotlin.random.Random

/**
 * Strategy for reconnection attempts with exponential backoff and jitter.
 *
 * @property initialDelayMs The initial delay before the first reconnection attempt, in milliseconds.
 * @property maxDelayMs The maximum delay between reconnection attempts, in milliseconds.
 * @property maxAttempts The maximum number of reconnection attempts. 0 means unlimited.
 * @property backoffMultiplier The multiplier applied to the delay for each subsequent attempt.
 * @property jitterFactor The fraction of the delay to randomize (0.0 to 1.0).
 */
data class ReconnectionStrategy(
    val initialDelayMs: Long = 1_000L,
    val maxDelayMs: Long = 30_000L,
    val maxAttempts: Int = 0,
    val backoffMultiplier: Double = 2.0,
    val jitterFactor: Double = 0.1,
) {

    /**
     * Calculates the delay for the given attempt number.
     *
     * @param attempt The current attempt number (0-based).
     * @return The delay in milliseconds, or null if max attempts exceeded.
     */
    fun delayForAttempt(attempt: Int): Long? {
        if (maxAttempts > 0 && attempt >= maxAttempts) return null

        val baseDelay = initialDelayMs * backoffMultiplier.pow(attempt.toDouble())
        val cappedDelay = min(baseDelay.toLong(), maxDelayMs)
        val jitter = (cappedDelay * jitterFactor * Random.nextDouble()).toLong()
        return cappedDelay + jitter
    }
}
