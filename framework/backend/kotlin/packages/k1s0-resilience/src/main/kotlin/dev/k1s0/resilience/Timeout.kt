package dev.k1s0.resilience

import kotlinx.coroutines.withTimeout
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

/**
 * Coroutine-based timeout wrapper.
 *
 * @property duration The maximum duration to wait for the operation.
 */
public class Timeout(
    private val duration: Duration = 30.seconds,
) {
    /**
     * Executes the given block with a timeout.
     *
     * @param block The suspending operation to execute.
     * @return The result of the block.
     * @throws kotlinx.coroutines.TimeoutCancellationException if the timeout is exceeded.
     */
    public suspend fun <T> execute(block: suspend () -> T): T =
        withTimeout(duration) { block() }
}
