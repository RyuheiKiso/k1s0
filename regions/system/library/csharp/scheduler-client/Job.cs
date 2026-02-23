namespace K1s0.System.SchedulerClient;

public enum JobStatus
{
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

public abstract record Schedule;

public record CronSchedule(string Expression) : Schedule;

public record OneShotSchedule(DateTimeOffset RunAt) : Schedule;

public record IntervalSchedule(TimeSpan Interval) : Schedule;

public record JobRequest(
    string Name,
    Schedule Schedule,
    object? Payload = null,
    uint MaxRetries = 3,
    ulong TimeoutSecs = 60);

public record Job(
    string Id,
    string Name,
    Schedule Schedule,
    JobStatus Status,
    object? Payload,
    uint MaxRetries,
    ulong TimeoutSecs,
    DateTimeOffset CreatedAt,
    DateTimeOffset? NextRunAt = null);

public record JobFilter(JobStatus? Status = null, string? NamePrefix = null);

public record JobExecution(
    string Id,
    string JobId,
    DateTimeOffset StartedAt,
    DateTimeOffset? FinishedAt,
    string Result,
    string? Error = null);

public record JobCompletedEvent(
    string JobId,
    string ExecutionId,
    DateTimeOffset CompletedAt,
    string Result);
