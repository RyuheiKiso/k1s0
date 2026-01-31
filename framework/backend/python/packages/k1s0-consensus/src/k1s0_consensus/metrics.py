"""Prometheus metrics for consensus primitives."""

from __future__ import annotations

from prometheus_client import Counter, Gauge, Histogram


class LeaderMetrics:
    """Metrics for leader election operations.

    All metric names are prefixed with ``k1s0_leader_``.
    """

    def __init__(self) -> None:
        self.elections_total = Counter(
            "k1s0_leader_elections_total",
            "Total number of leader election attempts",
            ["result"],
        )
        self.renewals_total = Counter(
            "k1s0_leader_renewals_total",
            "Total number of lease renewal attempts",
            ["result"],
        )
        self.is_leader = Gauge(
            "k1s0_leader_is_leader",
            "Whether this node is currently the leader (1=yes, 0=no)",
        )
        self.lease_duration_seconds = Histogram(
            "k1s0_leader_lease_duration_seconds",
            "Duration of leadership tenures in seconds",
            buckets=(1, 5, 15, 30, 60, 120, 300, 600),
        )


class LockMetrics:
    """Metrics for distributed lock operations.

    All metric names are prefixed with ``k1s0_lock_``.
    """

    def __init__(self) -> None:
        self.acquisitions_total = Counter(
            "k1s0_lock_acquisitions_total",
            "Total number of lock acquisition attempts",
            ["result"],
        )
        self.releases_total = Counter(
            "k1s0_lock_releases_total",
            "Total number of lock releases",
        )
        self.held_count = Gauge(
            "k1s0_lock_held_count",
            "Number of locks currently held by this node",
        )
        self.wait_duration_seconds = Histogram(
            "k1s0_lock_wait_duration_seconds",
            "Time spent waiting to acquire a lock",
            buckets=(0.01, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10),
        )


class SagaMetrics:
    """Metrics for saga execution.

    All metric names are prefixed with ``k1s0_saga_``.
    """

    def __init__(self) -> None:
        self.executions_total = Counter(
            "k1s0_saga_executions_total",
            "Total number of saga executions",
            ["saga_name", "result"],
        )
        self.steps_total = Counter(
            "k1s0_saga_steps_total",
            "Total number of saga steps executed",
            ["saga_name", "step_name", "result"],
        )
        self.compensations_total = Counter(
            "k1s0_saga_compensations_total",
            "Total number of compensation steps executed",
            ["saga_name", "step_name", "result"],
        )
        self.dead_letters_total = Counter(
            "k1s0_saga_dead_letters_total",
            "Total number of saga instances moved to dead-letter queue",
            ["saga_name"],
        )
        self.duration_seconds = Histogram(
            "k1s0_saga_duration_seconds",
            "Duration of complete saga executions in seconds",
            ["saga_name"],
            buckets=(0.1, 0.5, 1, 2.5, 5, 10, 30, 60, 120),
        )
        self.active_count = Gauge(
            "k1s0_saga_active_count",
            "Number of currently executing sagas",
            ["saga_name"],
        )
