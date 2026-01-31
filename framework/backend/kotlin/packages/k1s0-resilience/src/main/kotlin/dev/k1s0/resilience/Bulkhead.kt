package dev.k1s0.resilience

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.sync.Semaphore

private val logger = KotlinLogging.logger {}

/**
 * Coroutine-based bulkhead that limits the number of concurrent executions.
 *
 * Prevents a single operation type from consuming all available resources.
 *
 * @property name Name for logging and metrics.
 * @property maxConcurrent Maximum number of concurrent executions.
 */
public class Bulkhead(
    private val name: String,
    private val maxConcurrent: Int = 10,
) {
    private val semaphore = Semaphore(maxConcurrent)

    /**
     * Executes the given block within the bulkhead limit.
     *
     * @param block The suspending operation to execute.
     * @return The result of the block.
     */
    public suspend fun <T> execute(block: suspend () -> T): T {
        semaphore.acquire()
        return try {
            block()
        } finally {
            semaphore.release()
        }
    }
}
