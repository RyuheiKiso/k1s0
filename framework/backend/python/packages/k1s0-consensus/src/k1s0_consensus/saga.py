"""Saga orchestrator for distributed transaction coordination."""

from __future__ import annotations

import asyncio
import json
import logging
import time
import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Generic, TypeVar

import asyncpg

from k1s0_consensus.config import SagaConfig
from k1s0_consensus.error import CompensationFailedError, DeadLetterError, SagaFailedError
from k1s0_consensus.metrics import SagaMetrics

logger = logging.getLogger("k1s0.consensus.saga")

C = TypeVar("C")


class BackoffStrategy(Enum):
    """Strategy for retry backoff calculation."""

    FIXED = "fixed"
    LINEAR = "linear"
    EXPONENTIAL = "exponential"


@dataclass(frozen=True)
class RetryPolicy:
    """Retry policy for saga steps.

    Attributes:
        max_retries: Maximum retry attempts.
        backoff_strategy: The backoff calculation strategy.
        initial_delay_ms: Initial delay before the first retry.
        max_delay_ms: Maximum delay between retries.
        multiplier: Multiplier for exponential/linear backoff.
    """

    max_retries: int = 3
    backoff_strategy: BackoffStrategy = BackoffStrategy.EXPONENTIAL
    initial_delay_ms: int = 100
    max_delay_ms: int = 30_000
    multiplier: float = 2.0

    def delay_ms(self, attempt: int) -> int:
        """Calculate the delay in milliseconds for a given attempt.

        Args:
            attempt: The zero-based retry attempt number.

        Returns:
            Delay in milliseconds, capped at max_delay_ms.
        """
        if self.backoff_strategy == BackoffStrategy.FIXED:
            return min(self.initial_delay_ms, self.max_delay_ms)
        if self.backoff_strategy == BackoffStrategy.LINEAR:
            return min(self.initial_delay_ms + int(self.initial_delay_ms * self.multiplier * attempt), self.max_delay_ms)
        # EXPONENTIAL
        return min(int(self.initial_delay_ms * (self.multiplier ** attempt)), self.max_delay_ms)


class SagaStatus(Enum):
    """Status of a saga instance."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    COMPENSATING = "compensating"
    FAILED = "failed"
    DEAD_LETTERED = "dead_lettered"


class SagaStep(ABC, Generic[C]):
    """Abstract base class for a single saga step.

    Type parameter C represents the context type shared between steps.
    """

    @property
    @abstractmethod
    def name(self) -> str:
        """The unique name of this step within the saga."""

    @abstractmethod
    async def execute(self, context: C) -> C:
        """Execute the forward action of this step.

        Args:
            context: The saga context, accumulated from prior steps.

        Returns:
            The updated context after execution.
        """

    @abstractmethod
    async def compensate(self, context: C) -> C:
        """Execute the compensation (rollback) action for this step.

        Args:
            context: The saga context at the time of compensation.

        Returns:
            The context after compensation.
        """


@dataclass
class SagaDefinition:
    """Definition of a saga as an ordered list of steps.

    Attributes:
        name: The name of this saga definition.
        steps: Ordered list of saga steps.
        retry_policy: Default retry policy for steps.
    """

    name: str
    steps: list[SagaStep[Any]] = field(default_factory=list)
    retry_policy: RetryPolicy = field(default_factory=RetryPolicy)


class SagaBuilder:
    """Fluent builder for constructing SagaDefinition instances.

    Example::

        saga = (
            SagaBuilder("order-saga")
            .step(ReserveInventoryStep())
            .step(ChargePaymentStep())
            .step(ShipOrderStep())
            .with_retry(RetryPolicy(max_retries=5))
            .build()
        )
    """

    def __init__(self, name: str) -> None:
        self._name = name
        self._steps: list[SagaStep[Any]] = []
        self._retry_policy = RetryPolicy()

    def step(self, saga_step: SagaStep[Any]) -> SagaBuilder:
        """Add a step to the saga.

        Args:
            saga_step: The step to append.

        Returns:
            The builder for chaining.
        """
        self._steps.append(saga_step)
        return self

    def with_retry(self, policy: RetryPolicy) -> SagaBuilder:
        """Set the retry policy for the saga.

        Args:
            policy: The retry policy to use.

        Returns:
            The builder for chaining.
        """
        self._retry_policy = policy
        return self

    def build(self) -> SagaDefinition:
        """Build the SagaDefinition.

        Returns:
            The constructed SagaDefinition.

        Raises:
            ValueError: If no steps have been added.
        """
        if not self._steps:
            msg = "Saga must have at least one step"
            raise ValueError(msg)
        return SagaDefinition(
            name=self._name,
            steps=list(self._steps),
            retry_policy=self._retry_policy,
        )


@dataclass(frozen=True)
class SagaInstance:
    """Persistent record of a saga execution.

    Attributes:
        saga_id: Unique identifier for this saga instance.
        saga_name: Name of the saga definition.
        status: Current execution status.
        current_step: Index of the current/last executed step.
        context_json: JSON-serialized saga context.
        created_at: Unix timestamp of creation.
        updated_at: Unix timestamp of last update.
        error_message: Error message if the saga failed.
    """

    saga_id: str
    saga_name: str
    status: SagaStatus
    current_step: int
    context_json: str
    created_at: float
    updated_at: float
    error_message: str | None = None


@dataclass(frozen=True)
class SagaResult:
    """Result of a saga execution.

    Attributes:
        saga_id: The saga instance identifier.
        status: Final status.
        context: The final context object.
        error: Error message if the saga failed.
    """

    saga_id: str
    status: SagaStatus
    context: Any
    error: str | None = None


class SagaOrchestrator:
    """Orchestrates saga execution with database persistence.

    Persists saga state to PostgreSQL for durability and supports
    resumption of interrupted sagas.

    Args:
        pool: An asyncpg connection pool.
        config: Saga configuration.
        metrics: Optional metrics collector.
    """

    _SQL_CREATE_INSTANCE_TABLE = """
        CREATE TABLE IF NOT EXISTS {table} (
            saga_id       TEXT PRIMARY KEY,
            saga_name     TEXT NOT NULL,
            status        TEXT NOT NULL,
            current_step  INTEGER NOT NULL DEFAULT 0,
            context_json  TEXT NOT NULL DEFAULT '{{}}',
            created_at    DOUBLE PRECISION NOT NULL,
            updated_at    DOUBLE PRECISION NOT NULL,
            error_message TEXT
        )
    """

    _SQL_CREATE_STEP_TABLE = """
        CREATE TABLE IF NOT EXISTS {table} (
            id          TEXT PRIMARY KEY,
            saga_id     TEXT NOT NULL,
            step_name   TEXT NOT NULL,
            step_index  INTEGER NOT NULL,
            status      TEXT NOT NULL,
            started_at  DOUBLE PRECISION,
            completed_at DOUBLE PRECISION,
            error_message TEXT
        )
    """

    _SQL_CREATE_DEAD_LETTER_TABLE = """
        CREATE TABLE IF NOT EXISTS {table} (
            saga_id       TEXT PRIMARY KEY,
            saga_name     TEXT NOT NULL,
            context_json  TEXT NOT NULL,
            error_message TEXT NOT NULL,
            created_at    DOUBLE PRECISION NOT NULL
        )
    """

    _SQL_INSERT_INSTANCE = """
        INSERT INTO {table} (saga_id, saga_name, status, current_step, context_json, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
    """

    _SQL_UPDATE_INSTANCE = """
        UPDATE {table}
        SET status = $1, current_step = $2, context_json = $3, updated_at = $4, error_message = $5
        WHERE saga_id = $6
    """

    _SQL_INSERT_STEP = """
        INSERT INTO {table} (id, saga_id, step_name, step_index, status, started_at)
        VALUES ($1, $2, $3, $4, $5, $6)
    """

    _SQL_UPDATE_STEP = """
        UPDATE {table}
        SET status = $1, completed_at = $2, error_message = $3
        WHERE id = $4
    """

    _SQL_INSERT_DEAD_LETTER = """
        INSERT INTO {table} (saga_id, saga_name, context_json, error_message, created_at)
        VALUES ($1, $2, $3, $4, $5)
    """

    _SQL_SELECT_DEAD_LETTERS = """
        SELECT saga_id, saga_name, context_json, error_message, created_at
        FROM {table}
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
    """

    _SQL_SELECT_INSTANCE = """
        SELECT saga_id, saga_name, status, current_step, context_json, created_at, updated_at, error_message
        FROM {table}
        WHERE saga_id = $1
    """

    def __init__(
        self,
        pool: asyncpg.Pool,  # type: ignore[type-arg]
        config: SagaConfig | None = None,
        metrics: SagaMetrics | None = None,
    ) -> None:
        self._pool = pool
        self._config = config or SagaConfig()
        self._metrics = metrics

    async def ensure_tables(self) -> None:
        """Create all saga-related tables if they do not exist."""
        async with self._pool.acquire() as conn:
            await conn.execute(self._SQL_CREATE_INSTANCE_TABLE.format(table=self._config.instance_table_name))
            await conn.execute(self._SQL_CREATE_STEP_TABLE.format(table=self._config.step_table_name))
            await conn.execute(self._SQL_CREATE_DEAD_LETTER_TABLE.format(table=self._config.dead_letter.table_name))

    async def execute(self, definition: SagaDefinition, initial_context: Any) -> SagaResult:
        """Execute a saga from start to finish.

        Runs each step in order. On failure, compensates completed steps
        in reverse order. If retries are exhausted, the saga is
        dead-lettered.

        Args:
            definition: The saga definition to execute.
            initial_context: The initial context passed to the first step.

        Returns:
            A SagaResult with the final status and context.

        Raises:
            SagaFailedError: If the saga fails and compensation succeeds.
            CompensationFailedError: If compensation itself fails.
            DeadLetterError: If retries are exhausted.
        """
        saga_id = uuid.uuid4().hex
        now = time.time()
        context = initial_context

        if self._metrics:
            self._metrics.active_count.labels(saga_name=definition.name).inc()

        start_time = time.time()

        # Persist initial state
        async with self._pool.acquire() as conn:
            await conn.execute(
                self._SQL_INSERT_INSTANCE.format(table=self._config.instance_table_name),
                saga_id,
                definition.name,
                SagaStatus.RUNNING.value,
                0,
                json.dumps(initial_context, default=str),
                now,
                now,
            )

        completed_steps: list[int] = []

        try:
            for idx, step in enumerate(definition.steps):
                step_id = uuid.uuid4().hex
                step_start = time.time()

                async with self._pool.acquire() as conn:
                    await conn.execute(
                        self._SQL_INSERT_STEP.format(table=self._config.step_table_name),
                        step_id,
                        saga_id,
                        step.name,
                        idx,
                        "running",
                        step_start,
                    )

                last_error: Exception | None = None
                for attempt in range(definition.retry_policy.max_retries + 1):
                    try:
                        context = await step.execute(context)
                        last_error = None
                        break
                    except Exception as exc:  # noqa: BLE001
                        last_error = exc
                        if attempt < definition.retry_policy.max_retries:
                            delay = definition.retry_policy.delay_ms(attempt) / 1000.0
                            await asyncio.sleep(delay)

                if last_error is not None:
                    # Record step failure
                    async with self._pool.acquire() as conn:
                        await conn.execute(
                            self._SQL_UPDATE_STEP.format(table=self._config.step_table_name),
                            "failed",
                            time.time(),
                            str(last_error),
                            step_id,
                        )

                    if self._metrics:
                        self._metrics.steps_total.labels(
                            saga_name=definition.name, step_name=step.name, result="failed"
                        ).inc()

                    # Dead-letter if retries exhausted
                    await self._dead_letter(saga_id, definition.name, context, str(last_error))

                    # Compensate completed steps
                    await self._compensate(definition, completed_steps, context, saga_id)

                    async with self._pool.acquire() as conn:
                        await conn.execute(
                            self._SQL_UPDATE_INSTANCE.format(table=self._config.instance_table_name),
                            SagaStatus.FAILED.value,
                            idx,
                            json.dumps(context, default=str),
                            time.time(),
                            str(last_error),
                            saga_id,
                        )

                    if self._metrics:
                        self._metrics.executions_total.labels(
                            saga_name=definition.name, result="failed"
                        ).inc()

                    raise SagaFailedError(
                        f"Saga {definition.name} failed at step {step.name}: {last_error}",
                        saga_id=saga_id,
                        failed_step=step.name,
                    )

                # Step succeeded
                async with self._pool.acquire() as conn:
                    await conn.execute(
                        self._SQL_UPDATE_STEP.format(table=self._config.step_table_name),
                        "completed",
                        time.time(),
                        None,
                        step_id,
                    )

                if self._metrics:
                    self._metrics.steps_total.labels(
                        saga_name=definition.name, step_name=step.name, result="success"
                    ).inc()

                completed_steps.append(idx)

                # Update instance progress
                async with self._pool.acquire() as conn:
                    await conn.execute(
                        self._SQL_UPDATE_INSTANCE.format(table=self._config.instance_table_name),
                        SagaStatus.RUNNING.value,
                        idx + 1,
                        json.dumps(context, default=str),
                        time.time(),
                        None,
                        saga_id,
                    )

            # All steps completed
            async with self._pool.acquire() as conn:
                await conn.execute(
                    self._SQL_UPDATE_INSTANCE.format(table=self._config.instance_table_name),
                    SagaStatus.COMPLETED.value,
                    len(definition.steps),
                    json.dumps(context, default=str),
                    time.time(),
                    None,
                    saga_id,
                )

            if self._metrics:
                self._metrics.executions_total.labels(saga_name=definition.name, result="success").inc()
                self._metrics.duration_seconds.labels(saga_name=definition.name).observe(time.time() - start_time)

            return SagaResult(saga_id=saga_id, status=SagaStatus.COMPLETED, context=context)

        finally:
            if self._metrics:
                self._metrics.active_count.labels(saga_name=definition.name).dec()

    async def resume(self, saga_id: str, definition: SagaDefinition) -> SagaResult:
        """Resume an interrupted saga from its last persisted state.

        Args:
            saga_id: The saga instance to resume.
            definition: The saga definition (must match the original).

        Returns:
            A SagaResult with the final status.

        Raises:
            ValueError: If the saga instance is not found.
        """
        async with self._pool.acquire() as conn:
            row = await conn.fetchrow(
                self._SQL_SELECT_INSTANCE.format(table=self._config.instance_table_name),
                saga_id,
            )

        if row is None:
            msg = f"Saga instance {saga_id} not found"
            raise ValueError(msg)

        status = SagaStatus(row["status"])
        if status in (SagaStatus.COMPLETED, SagaStatus.DEAD_LETTERED):
            return SagaResult(
                saga_id=saga_id,
                status=status,
                context=json.loads(row["context_json"]),
                error=row["error_message"],
            )

        current_step = row["current_step"]
        context = json.loads(row["context_json"])

        # Re-execute remaining steps by creating a sub-definition
        remaining_steps = definition.steps[current_step:]
        sub_definition = SagaDefinition(
            name=definition.name,
            steps=remaining_steps,
            retry_policy=definition.retry_policy,
        )

        return await self.execute(sub_definition, context)

    async def dead_letters(self, limit: int = 100, offset: int = 0) -> list[dict[str, Any]]:
        """Retrieve dead-lettered saga instances.

        Args:
            limit: Maximum number of entries to return.
            offset: Number of entries to skip.

        Returns:
            A list of dead-letter records as dictionaries.
        """
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(
                self._SQL_SELECT_DEAD_LETTERS.format(table=self._config.dead_letter.table_name),
                limit,
                offset,
            )

        return [dict(row) for row in rows]

    async def _compensate(
        self,
        definition: SagaDefinition,
        completed_indices: list[int],
        context: Any,
        saga_id: str,
    ) -> None:
        """Compensate completed steps in reverse order.

        Args:
            definition: The saga definition.
            completed_indices: Indices of steps that completed successfully.
            context: The current saga context.
            saga_id: The saga instance ID for logging.
        """
        async with self._pool.acquire() as conn:
            await conn.execute(
                self._SQL_UPDATE_INSTANCE.format(table=self._config.instance_table_name),
                SagaStatus.COMPENSATING.value,
                completed_indices[-1] if completed_indices else 0,
                json.dumps(context, default=str),
                time.time(),
                None,
                saga_id,
            )

        for idx in reversed(completed_indices):
            step = definition.steps[idx]
            try:
                context = await step.compensate(context)
                if self._metrics:
                    self._metrics.compensations_total.labels(
                        saga_name=definition.name, step_name=step.name, result="success"
                    ).inc()
                logger.info("Compensated step %s for saga %s", step.name, saga_id)
            except Exception as exc:
                if self._metrics:
                    self._metrics.compensations_total.labels(
                        saga_name=definition.name, step_name=step.name, result="failed"
                    ).inc()
                logger.exception("Compensation failed for step %s in saga %s", step.name, saga_id)
                raise CompensationFailedError(
                    f"Compensation failed at step {step.name}: {exc}",
                    saga_id=saga_id,
                    failed_step=step.name,
                    original_error=exc,
                ) from exc

    async def _dead_letter(self, saga_id: str, saga_name: str, context: Any, error_message: str) -> None:
        """Move a saga to the dead-letter table.

        Args:
            saga_id: The saga instance ID.
            saga_name: The saga definition name.
            context: The current context.
            error_message: The error that caused dead-lettering.
        """
        async with self._pool.acquire() as conn:
            await conn.execute(
                self._SQL_INSERT_DEAD_LETTER.format(table=self._config.dead_letter.table_name),
                saga_id,
                saga_name,
                json.dumps(context, default=str),
                error_message,
                time.time(),
            )

        if self._metrics:
            self._metrics.dead_letters_total.labels(saga_name=saga_name).inc()

        logger.warning("Saga %s (%s) moved to dead-letter queue: %s", saga_id, saga_name, error_message)
