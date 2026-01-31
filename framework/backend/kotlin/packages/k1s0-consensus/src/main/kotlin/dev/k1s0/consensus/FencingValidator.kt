package dev.k1s0.consensus

import io.github.oshai.kotlinlogging.KotlinLogging
import java.util.concurrent.atomic.AtomicLong

private val logger = KotlinLogging.logger {}

/**
 * Validates fence tokens to ensure monotonic increase.
 *
 * Fence tokens are used to prevent stale lock holders from performing
 * operations after their lock has been revoked. Each resource should
 * have its own [FencingValidator] instance.
 *
 * Thread-safe via [AtomicLong].
 */
public class FencingValidator {

    private val highestSeen = AtomicLong(0L)

    /**
     * Validates that the given fence token is greater than or equal to the
     * highest previously seen token.
     *
     * @param fenceToken The fence token to validate.
     * @return `true` if the token is valid (monotonically non-decreasing).
     * @throws ConsensusError.FenceTokenViolation if the token is stale.
     */
    public fun validate(fenceToken: Long): Boolean {
        val current = highestSeen.get()
        if (fenceToken < current) {
            logger.warn { "Fence token violation: expected >= $current but got $fenceToken" }
            throw ConsensusError.FenceTokenViolation(expectedMinimum = current, actual = fenceToken)
        }
        highestSeen.updateAndGet { maxOf(it, fenceToken) }
        logger.debug { "Fence token validated: $fenceToken (highest=$fenceToken)" }
        return true
    }

    /**
     * Returns the highest fence token seen so far.
     */
    public fun highestToken(): Long = highestSeen.get()

    /**
     * Resets the validator to its initial state.
     */
    public fun reset() {
        highestSeen.set(0L)
    }
}
