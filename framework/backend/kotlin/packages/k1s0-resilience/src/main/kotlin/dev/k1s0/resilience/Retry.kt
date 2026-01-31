package dev.k1s0.resilience

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.delay
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds

private val logger = KotlinLogging.logger {}

/**
 * Coroutine-based retry mechanism with exponential backoff.
 *
 * @property maxAttempts Maximum number of attempts (including the initial call).
 * @property initialDelay Initial delay before the first retry.
 * @property maxDelay Maximum delay between retries.
 * @property multiplier Backoff multiplier applied after each retry.
 */
public class Retry(
    private val maxAttempts: Int = 3,
    private val initialDelay: Duration = 100.milliseconds,
    private val maxDelay: Duration = 5000.milliseconds,
    private val multiplier: Double = 2.0,
) {
    /**
     * Executes the given block with retry logic.
     *
     * @param block The suspending operation to retry on failure.
     * @return The result of the block.
     * @throws Exception The last exception if all attempts fail.
     */
    public suspend fun <T> execute(block: suspend () -> T): T {
        var currentDelay = initialDelay
        var lastException: Exception? = null

        repeat(maxAttempts) { attempt ->
            try {
                return block()
            } catch (e: Exception) {
                lastException = e
                if (attempt < maxAttempts - 1) {
                    logger.debug { "Retry attempt ${attempt + 1}/$maxAttempts failed: ${e.message}" }
                    delay(currentDelay)
                    currentDelay = (currentDelay * multiplier).coerceAtMost(maxDelay)
                }
            }
        }

        throw lastException!!
    }
}
