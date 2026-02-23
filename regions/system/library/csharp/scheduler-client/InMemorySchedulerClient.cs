namespace K1s0.System.SchedulerClient;

public class InMemorySchedulerClient : ISchedulerClient
{
    private readonly Dictionary<string, Job> _jobs = new();
    private int _seq;

    public IReadOnlyDictionary<string, Job> Jobs => _jobs;

    public Task<Job> CreateJobAsync(JobRequest req, CancellationToken ct = default)
    {
        _seq++;
        var id = $"job-{_seq:D3}";
        var job = new Job(
            id,
            req.Name,
            req.Schedule,
            JobStatus.Pending,
            req.Payload,
            req.MaxRetries,
            req.TimeoutSecs,
            DateTimeOffset.UtcNow);
        _jobs[id] = job;
        return Task.FromResult(job);
    }

    public Task CancelJobAsync(string jobId, CancellationToken ct = default)
    {
        if (!_jobs.TryGetValue(jobId, out var job))
        {
            throw new SchedulerException($"Job not found: {jobId}", "JOB_NOT_FOUND");
        }

        _jobs[jobId] = job with { Status = JobStatus.Cancelled };
        return Task.CompletedTask;
    }

    public Task PauseJobAsync(string jobId, CancellationToken ct = default)
    {
        if (!_jobs.TryGetValue(jobId, out var job))
        {
            throw new SchedulerException($"Job not found: {jobId}", "JOB_NOT_FOUND");
        }

        _jobs[jobId] = job with { Status = JobStatus.Paused };
        return Task.CompletedTask;
    }

    public Task ResumeJobAsync(string jobId, CancellationToken ct = default)
    {
        if (!_jobs.TryGetValue(jobId, out var job))
        {
            throw new SchedulerException($"Job not found: {jobId}", "JOB_NOT_FOUND");
        }

        _jobs[jobId] = job with { Status = JobStatus.Pending };
        return Task.CompletedTask;
    }

    public Task<Job> GetJobAsync(string jobId, CancellationToken ct = default)
    {
        if (!_jobs.TryGetValue(jobId, out var job))
        {
            throw new SchedulerException($"Job not found: {jobId}", "JOB_NOT_FOUND");
        }

        return Task.FromResult(job);
    }

    public Task<IReadOnlyList<Job>> ListJobsAsync(JobFilter? filter = null, CancellationToken ct = default)
    {
        IEnumerable<Job> result = _jobs.Values;
        if (filter?.Status is not null)
        {
            result = result.Where(j => j.Status == filter.Status);
        }

        if (!string.IsNullOrEmpty(filter?.NamePrefix))
        {
            result = result.Where(j => j.Name.StartsWith(filter!.NamePrefix!, StringComparison.Ordinal));
        }

        IReadOnlyList<Job> list = result.ToList();
        return Task.FromResult(list);
    }

    public Task<IReadOnlyList<JobExecution>> GetExecutionsAsync(string jobId, CancellationToken ct = default)
    {
        IReadOnlyList<JobExecution> empty = Array.Empty<JobExecution>();
        return Task.FromResult(empty);
    }

    public ValueTask DisposeAsync()
    {
        _jobs.Clear();
        return ValueTask.CompletedTask;
    }
}
