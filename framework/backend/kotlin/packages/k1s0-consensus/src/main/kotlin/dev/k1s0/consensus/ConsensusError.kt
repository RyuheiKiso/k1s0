package dev.k1s0.consensus

import dev.k1s0.error.ErrorCode
import dev.k1s0.error.K1s0Exception

/**
 * Sealed class representing all consensus-related errors.
 */
public sealed class ConsensusError(
    serviceErrorCode: String,
    detail: String,
    cause: Throwable? = null,
) : K1s0Exception(ErrorCode.INTERNAL, serviceErrorCode, detail, cause = cause) {

    /** The leader lease has expired and the node is no longer the leader. */
    public class LeaseExpired(
        detail: String = "Leader lease has expired",
        cause: Throwable? = null,
    ) : ConsensusError("consensus.lease_expired", detail, cause)

    /** A distributed lock acquisition timed out. */
    public class LockTimeout(
        detail: String = "Lock acquisition timed out",
        cause: Throwable? = null,
    ) : ConsensusError("consensus.lock_timeout", detail, cause)

    /** A fence token violation was detected, indicating a stale lock holder. */
    public class FenceTokenViolation(
        public val expectedMinimum: Long,
        public val actual: Long,
        cause: Throwable? = null,
    ) : ConsensusError(
        "consensus.fence_token_violation",
        "Fence token violation: expected >= $expectedMinimum but got $actual",
        cause,
    )

    /** A saga execution failed. */
    public class SagaFailed(
        public val sagaId: String,
        public val failedStep: String,
        detail: String = "Saga '$sagaId' failed at step '$failedStep'",
        cause: Throwable? = null,
    ) : ConsensusError("consensus.saga_failed", detail, cause)

    /** Compensation for a saga step failed. */
    public class CompensationFailed(
        public val sagaId: String,
        public val failedStep: String,
        detail: String = "Compensation failed for saga '$sagaId' at step '$failedStep'",
        cause: Throwable? = null,
    ) : ConsensusError("consensus.compensation_failed", detail, cause)

    /** A saga has been moved to the dead letter queue after exhausting retries. */
    public class DeadLetter(
        public val sagaId: String,
        detail: String = "Saga '$sagaId' moved to dead letter queue",
        cause: Throwable? = null,
    ) : ConsensusError("consensus.dead_letter", detail, cause)
}
