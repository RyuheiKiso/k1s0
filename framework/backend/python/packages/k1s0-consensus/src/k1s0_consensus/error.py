"""Consensus error hierarchy for k1s0."""

from __future__ import annotations


class ConsensusError(Exception):
    """Base exception for all consensus errors.

    Attributes:
        message: Human-readable description of the error.
    """

    def __init__(self, message: str) -> None:
        self.message = message
        super().__init__(message)


class LeaseExpiredError(ConsensusError):
    """Raised when a leader lease has expired and cannot be renewed.

    This indicates the node lost leadership and must stop acting
    as the leader immediately.
    """


class LockTimeoutError(ConsensusError):
    """Raised when acquiring a distributed lock exceeds the timeout.

    The caller should retry or abort the operation that requires
    exclusive access.
    """


class FenceTokenViolationError(ConsensusError):
    """Raised when a stale fence token is presented.

    A fence token violation means a previous lock holder is attempting
    to perform an operation after its lock was superseded.
    """


class SagaFailedError(ConsensusError):
    """Raised when a saga fails to complete all steps.

    Attributes:
        saga_id: The unique identifier of the failed saga instance.
        failed_step: The name of the step that failed.
    """

    def __init__(self, message: str, *, saga_id: str, failed_step: str) -> None:
        super().__init__(message)
        self.saga_id = saga_id
        self.failed_step = failed_step


class CompensationFailedError(ConsensusError):
    """Raised when saga compensation (rollback) fails.

    This is a critical error: the system is in an inconsistent state
    and manual intervention may be required.

    Attributes:
        saga_id: The unique identifier of the saga instance.
        failed_step: The compensation step that failed.
        original_error: The error that triggered compensation.
    """

    def __init__(
        self,
        message: str,
        *,
        saga_id: str,
        failed_step: str,
        original_error: Exception | None = None,
    ) -> None:
        super().__init__(message)
        self.saga_id = saga_id
        self.failed_step = failed_step
        self.original_error = original_error


class DeadLetterError(ConsensusError):
    """Raised when a saga instance is moved to the dead-letter queue.

    Dead-lettered sagas have exhausted all retry attempts and require
    manual inspection or reprocessing.

    Attributes:
        saga_id: The unique identifier of the dead-lettered saga.
    """

    def __init__(self, message: str, *, saga_id: str) -> None:
        super().__init__(message)
        self.saga_id = saga_id
