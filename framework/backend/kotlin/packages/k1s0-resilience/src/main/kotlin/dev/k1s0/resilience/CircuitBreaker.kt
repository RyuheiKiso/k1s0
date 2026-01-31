package dev.k1s0.resilience

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeSource

private val logger = KotlinLogging.logger {}

/**
 * Coroutine-based circuit breaker for fault tolerance.
 *
 * Transitions between CLOSED, OPEN, and HALF_OPEN states based on
 * failure thresholds and recovery timeouts.
 *
 * @property name Name for logging and metrics.
 * @property failureThreshold Number of failures before opening the circuit.
 * @property resetTimeout Duration to wait before transitioning from OPEN to HALF_OPEN.
 */
public class CircuitBreaker(
    private val name: String,
    private val failureThreshold: Int = 5,
    private val resetTimeout: Duration = 30.seconds,
) {
    /** Circuit breaker states. */
    public enum class State { CLOSED, OPEN, HALF_OPEN }

    private val mutex = Mutex()
    private var state: State = State.CLOSED
    private var failureCount: Int = 0
    private var lastFailureTime: TimeSource.Monotonic.ValueTimeMark? = null

    /** Returns the current state of the circuit breaker. */
    public suspend fun currentState(): State = mutex.withLock { state }

    /**
     * Executes the given block with circuit breaker protection.
     *
     * @param block The suspending operation to protect.
     * @return The result of the block.
     * @throws CircuitBreakerOpenException if the circuit is open.
     */
    public suspend fun <T> execute(block: suspend () -> T): T {
        mutex.withLock {
            when (state) {
                State.OPEN -> {
                    val elapsed = lastFailureTime?.elapsedNow() ?: Duration.ZERO
                    if (elapsed >= resetTimeout) {
                        state = State.HALF_OPEN
                        logger.info { "Circuit breaker '$name' transitioning to HALF_OPEN" }
                    } else {
                        throw CircuitBreakerOpenException(name)
                    }
                }
                else -> { /* proceed */ }
            }
        }

        return try {
            val result = block()
            mutex.withLock {
                if (state == State.HALF_OPEN) {
                    state = State.CLOSED
                    failureCount = 0
                    logger.info { "Circuit breaker '$name' closed" }
                }
            }
            result
        } catch (e: Exception) {
            mutex.withLock {
                failureCount++
                lastFailureTime = TimeSource.Monotonic.markNow()
                if (failureCount >= failureThreshold) {
                    state = State.OPEN
                    logger.warn { "Circuit breaker '$name' opened after $failureCount failures" }
                }
            }
            throw e
        }
    }
}

/** Exception thrown when a circuit breaker is in the OPEN state. */
public class CircuitBreakerOpenException(
    public val circuitName: String,
) : RuntimeException("Circuit breaker '$circuitName' is open")
