package dev.k1s0.consensus

import java.time.Instant

/**
 * Status of a saga execution.
 */
public enum class SagaStatus {
    /** The saga is currently executing steps. */
    RUNNING,

    /** All steps completed successfully. */
    COMPLETED,

    /** A step failed and compensation is in progress. */
    COMPENSATING,

    /** Compensation completed after a failure. */
    COMPENSATED,

    /** The saga failed and could not be compensated. */
    FAILED,

    /** The saga has been moved to the dead letter queue. */
    DEAD_LETTER,
}

/**
 * Persistent representation of a saga instance.
 *
 * @property sagaId Unique identifier for this saga execution.
 * @property sagaName The name of the saga definition.
 * @property status Current execution status.
 * @property currentStep The index of the current or failed step.
 * @property context Serialized context data.
 * @property error Error message if the saga failed.
 * @property retryCount Number of retries attempted.
 * @property createdAt When the saga was created.
 * @property updatedAt When the saga was last updated.
 */
public data class SagaInstance(
    val sagaId: String,
    val sagaName: String,
    val status: SagaStatus,
    val currentStep: Int,
    val context: String,
    val error: String? = null,
    val retryCount: Int = 0,
    val createdAt: Instant,
    val updatedAt: Instant,
)

/**
 * Result of a saga execution.
 *
 * @property sagaId Unique identifier for this saga execution.
 * @property status Final status of the saga.
 * @property context The final context after execution.
 * @property error Error message if the saga failed.
 * @property completedSteps List of step names that completed successfully.
 * @property compensatedSteps List of step names that were compensated.
 */
public data class SagaResult(
    val sagaId: String,
    val status: SagaStatus,
    val context: Map<String, Any>,
    val error: String? = null,
    val completedSteps: List<String> = emptyList(),
    val compensatedSteps: List<String> = emptyList(),
)
