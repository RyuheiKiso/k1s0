namespace K1s0.System.SchedulerClient;

public interface ISchedulerClient : IAsyncDisposable
{
    Task<Job> CreateJobAsync(JobRequest req, CancellationToken ct = default);

    Task CancelJobAsync(string jobId, CancellationToken ct = default);

    Task PauseJobAsync(string jobId, CancellationToken ct = default);

    Task ResumeJobAsync(string jobId, CancellationToken ct = default);

    Task<Job> GetJobAsync(string jobId, CancellationToken ct = default);

    Task<IReadOnlyList<Job>> ListJobsAsync(JobFilter? filter = null, CancellationToken ct = default);

    Task<IReadOnlyList<JobExecution>> GetExecutionsAsync(string jobId, CancellationToken ct = default);
}
