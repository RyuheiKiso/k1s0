"""Scheduler client for job scheduling operations."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from enum import Enum
from typing import Any


class JobStatus(str, Enum):
    """Job status types."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    PAUSED = "paused"
    CANCELLED = "cancelled"


@dataclass
class Schedule:
    """Job schedule configuration."""

    type: str  # "cron", "one_shot", "interval"
    cron: str | None = None
    one_shot: datetime | None = None
    interval: timedelta | None = None

    @classmethod
    def cron(cls, expression: str) -> Schedule:
        return cls(type="cron", cron=expression)

    @classmethod
    def one_shot(cls, run_at: datetime) -> Schedule:
        return cls(type="one_shot", one_shot=run_at)

    @classmethod
    def interval_of(cls, duration: timedelta) -> Schedule:
        return cls(type="interval", interval=duration)


@dataclass
class JobRequest:
    """Job creation request."""

    name: str
    schedule: Schedule
    payload: dict[str, Any] = field(default_factory=dict)
    max_retries: int = 3
    timeout_secs: int = 60


@dataclass
class Job:
    """Job information."""

    id: str
    name: str
    schedule: Schedule
    status: JobStatus
    payload: dict[str, Any]
    max_retries: int
    timeout_secs: int
    created_at: datetime
    next_run_at: datetime | None = None


@dataclass
class JobFilter:
    """Job list filter."""

    status: JobStatus | None = None
    name_prefix: str | None = None


@dataclass
class JobExecution:
    """Job execution history."""

    id: str
    job_id: str
    started_at: datetime
    finished_at: datetime | None
    result: str
    error: str | None = None


@dataclass
class JobCompletedEvent:
    """Job completed event from Kafka."""

    job_id: str
    execution_id: str
    completed_at: datetime
    result: str


class SchedulerError(Exception):
    """Scheduler error."""

    def __init__(self, message: str, code: str) -> None:
        super().__init__(message)
        self.code = code


class SchedulerClient(ABC):
    """Abstract scheduler client."""

    @abstractmethod
    async def create_job(self, request: JobRequest) -> Job: ...

    @abstractmethod
    async def cancel_job(self, job_id: str) -> None: ...

    @abstractmethod
    async def pause_job(self, job_id: str) -> None: ...

    @abstractmethod
    async def resume_job(self, job_id: str) -> None: ...

    @abstractmethod
    async def get_job(self, job_id: str) -> Job: ...

    @abstractmethod
    async def list_jobs(self, filter: JobFilter | None = None) -> list[Job]: ...

    @abstractmethod
    async def get_executions(self, job_id: str) -> list[JobExecution]: ...


class InMemorySchedulerClient(SchedulerClient):
    """In-memory scheduler client for testing."""

    def __init__(self) -> None:
        self._jobs: dict[str, Job] = {}
        self._seq = 0

    @property
    def jobs(self) -> dict[str, Job]:
        """Get a copy of all jobs."""
        return dict(self._jobs)

    async def create_job(self, request: JobRequest) -> Job:
        self._seq += 1
        job_id = f"job-{self._seq:03d}"
        job = Job(
            id=job_id,
            name=request.name,
            schedule=request.schedule,
            status=JobStatus.PENDING,
            payload=request.payload,
            max_retries=request.max_retries,
            timeout_secs=request.timeout_secs,
            created_at=datetime.now(timezone.utc),
        )
        self._jobs[job_id] = job
        return job

    async def cancel_job(self, job_id: str) -> None:
        if job_id not in self._jobs:
            raise SchedulerError(f"Job not found: {job_id}", "JOB_NOT_FOUND")
        self._jobs[job_id].status = JobStatus.CANCELLED

    async def pause_job(self, job_id: str) -> None:
        if job_id not in self._jobs:
            raise SchedulerError(f"Job not found: {job_id}", "JOB_NOT_FOUND")
        self._jobs[job_id].status = JobStatus.PAUSED

    async def resume_job(self, job_id: str) -> None:
        if job_id not in self._jobs:
            raise SchedulerError(f"Job not found: {job_id}", "JOB_NOT_FOUND")
        self._jobs[job_id].status = JobStatus.PENDING

    async def get_job(self, job_id: str) -> Job:
        if job_id not in self._jobs:
            raise SchedulerError(f"Job not found: {job_id}", "JOB_NOT_FOUND")
        return self._jobs[job_id]

    async def list_jobs(self, filter: JobFilter | None = None) -> list[Job]:
        result = list(self._jobs.values())
        if filter and filter.status is not None:
            result = [j for j in result if j.status == filter.status]
        if filter and filter.name_prefix:
            result = [j for j in result if j.name.startswith(filter.name_prefix)]
        return result

    async def get_executions(self, job_id: str) -> list[JobExecution]:
        return []


class GrpcSchedulerClient(SchedulerClient):
    """gRPC scheduler client (stub for future implementation)."""

    def __init__(self, server_url: str) -> None:
        self._server_url = server_url

    async def create_job(self, request: JobRequest) -> Job:
        raise NotImplementedError("gRPC client not yet implemented")

    async def cancel_job(self, job_id: str) -> None:
        raise NotImplementedError("gRPC client not yet implemented")

    async def pause_job(self, job_id: str) -> None:
        raise NotImplementedError("gRPC client not yet implemented")

    async def resume_job(self, job_id: str) -> None:
        raise NotImplementedError("gRPC client not yet implemented")

    async def get_job(self, job_id: str) -> Job:
        raise NotImplementedError("gRPC client not yet implemented")

    async def list_jobs(self, filter: JobFilter | None = None) -> list[Job]:
        raise NotImplementedError("gRPC client not yet implemented")

    async def get_executions(self, job_id: str) -> list[JobExecution]:
        raise NotImplementedError("gRPC client not yet implemented")
