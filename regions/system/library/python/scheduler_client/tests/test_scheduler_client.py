"""scheduler_client library unit tests."""

from datetime import datetime, timedelta, timezone

import pytest

from k1s0_scheduler_client import (
    InMemorySchedulerClient,
    JobCompletedEvent,
    JobExecution,
    JobFilter,
    JobRequest,
    JobStatus,
    Schedule,
    SchedulerError,
)


async def test_create_job() -> None:
    client = InMemorySchedulerClient()
    req = JobRequest(
        name="daily-report",
        schedule=Schedule.cron("0 2 * * *"),
        payload={"report_type": "daily"},
        max_retries=3,
        timeout_secs=300,
    )
    job = await client.create_job(req)
    assert job.id == "job-001"
    assert job.name == "daily-report"
    assert job.status == JobStatus.PENDING


async def test_cancel_job() -> None:
    client = InMemorySchedulerClient()
    job = await client.create_job(
        JobRequest(name="test", schedule=Schedule.cron("* * * * *"))
    )
    await client.cancel_job(job.id)
    got = await client.get_job(job.id)
    assert got.status == JobStatus.CANCELLED


async def test_pause_and_resume() -> None:
    client = InMemorySchedulerClient()
    job = await client.create_job(
        JobRequest(name="test", schedule=Schedule.cron("* * * * *"))
    )
    await client.pause_job(job.id)
    assert (await client.get_job(job.id)).status == JobStatus.PAUSED

    await client.resume_job(job.id)
    assert (await client.get_job(job.id)).status == JobStatus.PENDING


async def test_get_job_not_found() -> None:
    client = InMemorySchedulerClient()
    with pytest.raises(SchedulerError):
        await client.get_job("nonexistent")


async def test_list_jobs_with_status_filter() -> None:
    client = InMemorySchedulerClient()
    await client.create_job(
        JobRequest(name="job-a", schedule=Schedule.cron("* * * * *"))
    )
    job_b = await client.create_job(
        JobRequest(name="job-b", schedule=Schedule.cron("* * * * *"))
    )
    await client.pause_job(job_b.id)

    paused = await client.list_jobs(JobFilter(status=JobStatus.PAUSED))
    assert len(paused) == 1
    assert paused[0].status == JobStatus.PAUSED


async def test_list_jobs_with_name_prefix() -> None:
    client = InMemorySchedulerClient()
    await client.create_job(
        JobRequest(name="daily-report", schedule=Schedule.cron("0 2 * * *"))
    )
    await client.create_job(
        JobRequest(name="weekly-report", schedule=Schedule.cron("0 2 * * 0"))
    )

    daily = await client.list_jobs(JobFilter(name_prefix="daily"))
    assert len(daily) == 1
    assert daily[0].name == "daily-report"


async def test_get_executions_empty() -> None:
    client = InMemorySchedulerClient()
    execs = await client.get_executions("job-001")
    assert len(execs) == 0


async def test_cancel_job_not_found() -> None:
    client = InMemorySchedulerClient()
    with pytest.raises(SchedulerError):
        await client.cancel_job("nonexistent")


async def test_jobs_property() -> None:
    client = InMemorySchedulerClient()
    await client.create_job(
        JobRequest(name="test", schedule=Schedule.cron("* * * * *"))
    )
    jobs = client.jobs
    assert len(jobs) == 1


async def test_schedule_variants() -> None:
    cron = Schedule.cron("0 * * * *")
    assert cron.type == "cron"
    assert cron.cron == "0 * * * *"

    run_at = datetime.now(timezone.utc)
    one_shot = Schedule.one_shot(run_at)
    assert one_shot.type == "one_shot"
    assert one_shot.one_shot == run_at

    interval = Schedule.interval_of(timedelta(minutes=10))
    assert interval.type == "interval"
    assert interval.interval == timedelta(minutes=10)


async def test_job_status_values() -> None:
    statuses = list(JobStatus)
    assert len(statuses) == 6


async def test_job_completed_event() -> None:
    event = JobCompletedEvent(
        job_id="job-1",
        execution_id="exec-1",
        completed_at=datetime.now(timezone.utc),
        result="success",
    )
    assert event.job_id == "job-1"
    assert event.result == "success"


async def test_job_execution_dataclass() -> None:
    exec_ = JobExecution(
        id="exec-1",
        job_id="job-1",
        started_at=datetime.now(timezone.utc),
        finished_at=None,
        result="running",
    )
    assert exec_.id == "exec-1"
    assert exec_.error is None


async def test_scheduler_error() -> None:
    err = SchedulerError("not found", "JOB_NOT_FOUND")
    assert str(err) == "not found"
    assert err.code == "JOB_NOT_FOUND"


async def test_list_jobs_no_filter() -> None:
    client = InMemorySchedulerClient()
    await client.create_job(
        JobRequest(name="a", schedule=Schedule.cron("* * * * *"))
    )
    await client.create_job(
        JobRequest(name="b", schedule=Schedule.cron("* * * * *"))
    )
    all_jobs = await client.list_jobs()
    assert len(all_jobs) == 2
