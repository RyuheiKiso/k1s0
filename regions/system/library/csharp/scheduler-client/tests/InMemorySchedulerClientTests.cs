using K1s0.System.SchedulerClient;

namespace K1s0.System.SchedulerClient.Tests;

public class InMemorySchedulerClientTests
{
    [Fact]
    public async Task CreateJob_ReturnsJob()
    {
        var client = new InMemorySchedulerClient();
        var req = new JobRequest("daily-report", new CronSchedule("0 2 * * *"), new { ReportType = "daily" }, 3, 300);
        var job = await client.CreateJobAsync(req);

        Assert.Equal("job-001", job.Id);
        Assert.Equal("daily-report", job.Name);
        Assert.Equal(JobStatus.Pending, job.Status);
    }

    [Fact]
    public async Task CancelJob_UpdatesStatus()
    {
        var client = new InMemorySchedulerClient();
        var job = await client.CreateJobAsync(new JobRequest("test", new CronSchedule("* * * * *")));
        await client.CancelJobAsync(job.Id);

        var got = await client.GetJobAsync(job.Id);
        Assert.Equal(JobStatus.Cancelled, got.Status);
    }

    [Fact]
    public async Task PauseAndResume_WorkCorrectly()
    {
        var client = new InMemorySchedulerClient();
        var job = await client.CreateJobAsync(new JobRequest("test", new CronSchedule("* * * * *")));

        await client.PauseJobAsync(job.Id);
        Assert.Equal(JobStatus.Paused, (await client.GetJobAsync(job.Id)).Status);

        await client.ResumeJobAsync(job.Id);
        Assert.Equal(JobStatus.Pending, (await client.GetJobAsync(job.Id)).Status);
    }

    [Fact]
    public async Task GetJob_NotFound_Throws()
    {
        var client = new InMemorySchedulerClient();
        await Assert.ThrowsAsync<SchedulerException>(() => client.GetJobAsync("nonexistent"));
    }

    [Fact]
    public async Task ListJobs_WithStatusFilter()
    {
        var client = new InMemorySchedulerClient();
        await client.CreateJobAsync(new JobRequest("job-a", new CronSchedule("* * * * *")));
        var jobB = await client.CreateJobAsync(new JobRequest("job-b", new CronSchedule("* * * * *")));
        await client.PauseJobAsync(jobB.Id);

        var paused = await client.ListJobsAsync(new JobFilter(Status: JobStatus.Paused));
        Assert.Single(paused);
        Assert.Equal(JobStatus.Paused, paused[0].Status);
    }

    [Fact]
    public async Task ListJobs_WithNamePrefixFilter()
    {
        var client = new InMemorySchedulerClient();
        await client.CreateJobAsync(new JobRequest("daily-report", new CronSchedule("0 2 * * *")));
        await client.CreateJobAsync(new JobRequest("weekly-report", new CronSchedule("0 2 * * 0")));

        var daily = await client.ListJobsAsync(new JobFilter(NamePrefix: "daily"));
        Assert.Single(daily);
        Assert.Equal("daily-report", daily[0].Name);
    }

    [Fact]
    public async Task GetExecutions_ReturnsEmpty()
    {
        var client = new InMemorySchedulerClient();
        var execs = await client.GetExecutionsAsync("job-001");
        Assert.Empty(execs);
    }

    [Fact]
    public async Task CancelJob_NotFound_Throws()
    {
        var client = new InMemorySchedulerClient();
        Assert.Throws<SchedulerException>(() => client.CancelJobAsync("none").GetAwaiter().GetResult());
    }

    [Fact]
    public void Schedule_Variants()
    {
        Schedule cron = new CronSchedule("0 * * * *");
        Assert.IsType<CronSchedule>(cron);

        Schedule oneShot = new OneShotSchedule(DateTimeOffset.UtcNow);
        Assert.IsType<OneShotSchedule>(oneShot);

        Schedule interval = new IntervalSchedule(TimeSpan.FromMinutes(10));
        Assert.IsType<IntervalSchedule>(interval);
    }

    [Fact]
    public void JobStatus_AllVariants()
    {
        var values = Enum.GetValues<JobStatus>();
        Assert.Equal(6, values.Length);
    }

    [Fact]
    public async Task DisposeAsync_ClearsJobs()
    {
        var client = new InMemorySchedulerClient();
        await client.CreateJobAsync(new JobRequest("test", new CronSchedule("* * * * *")));
        await client.DisposeAsync();
        Assert.Empty(client.Jobs);
    }
}
