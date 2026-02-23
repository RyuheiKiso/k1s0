import Testing
@testable import K1s0SchedulerClient

@Suite("SchedulerClient Tests")
struct SchedulerClientTests {
    @Test("ジョブを作成できること")
    func testCreateJob() async throws {
        let client = InMemorySchedulerClient()
        let req = JobRequest(
            name: "daily-report",
            schedule: .cron("0 2 * * *"),
            payload: ["report_type": "daily"],
            maxRetries: 3,
            timeoutSecs: 300
        )
        let job = try await client.createJob(req)
        #expect(job.id == "job-001")
        #expect(job.name == "daily-report")
        #expect(job.status == .pending)
    }

    @Test("ジョブをキャンセルできること")
    func testCancelJob() async throws {
        let client = InMemorySchedulerClient()
        let job = try await client.createJob(JobRequest(
            name: "test", schedule: .cron("* * * * *")
        ))
        try await client.cancelJob(jobId: job.id)
        let got = try await client.getJob(jobId: job.id)
        #expect(got.status == .cancelled)
    }

    @Test("ジョブを一時停止・再開できること")
    func testPauseResume() async throws {
        let client = InMemorySchedulerClient()
        let job = try await client.createJob(JobRequest(
            name: "test", schedule: .cron("* * * * *")
        ))

        try await client.pauseJob(jobId: job.id)
        #expect(try await client.getJob(jobId: job.id).status == .paused)

        try await client.resumeJob(jobId: job.id)
        #expect(try await client.getJob(jobId: job.id).status == .pending)
    }

    @Test("存在しないジョブ取得でエラーになること")
    func testGetJobNotFound() async {
        let client = InMemorySchedulerClient()
        do {
            _ = try await client.getJob(jobId: "nonexistent")
            Issue.record("Expected error")
        } catch {
            #expect(error is SchedulerError)
        }
    }

    @Test("ステータスフィルターでジョブ一覧を取得できること")
    func testListJobsWithFilter() async throws {
        let client = InMemorySchedulerClient()
        _ = try await client.createJob(JobRequest(name: "job-a", schedule: .cron("* * * * *")))
        let jobB = try await client.createJob(JobRequest(name: "job-b", schedule: .cron("* * * * *")))
        try await client.pauseJob(jobId: jobB.id)

        let paused = try await client.listJobs(filter: JobFilter(status: .paused))
        #expect(paused.count == 1)
        #expect(paused[0].status == .paused)
    }

    @Test("名前プレフィックスでフィルターできること")
    func testListJobsWithNamePrefix() async throws {
        let client = InMemorySchedulerClient()
        _ = try await client.createJob(JobRequest(name: "daily-report", schedule: .cron("0 2 * * *")))
        _ = try await client.createJob(JobRequest(name: "weekly-report", schedule: .cron("0 2 * * 0")))

        let daily = try await client.listJobs(filter: JobFilter(namePrefix: "daily"))
        #expect(daily.count == 1)
        #expect(daily[0].name == "daily-report")
    }

    @Test("実行履歴が空で返ること")
    func testGetExecutions() async throws {
        let client = InMemorySchedulerClient()
        let execs = try await client.getExecutions(jobId: "job-001")
        #expect(execs.isEmpty)
    }

    @Test("Scheduleの各バリアントが作成できること")
    func testScheduleVariants() {
        let cron = Schedule.cron("0 * * * *")
        if case .cron = cron {} else {
            Issue.record("Expected cron schedule")
        }

        let oneShot = Schedule.oneShot(Date())
        if case .oneShot = oneShot {} else {
            Issue.record("Expected oneShot schedule")
        }

        let interval = Schedule.interval(.seconds(600))
        if case .interval = interval {} else {
            Issue.record("Expected interval schedule")
        }
    }

    @Test("JobCompletedEventが作成できること")
    func testJobCompletedEvent() {
        let event = JobCompletedEvent(
            jobId: "job-1",
            executionId: "exec-1",
            completedAt: Date(),
            result: "success"
        )
        #expect(event.jobId == "job-1")
        #expect(event.result == "success")
    }

    @Test("全JobStatusバリアント")
    func testAllJobStatuses() {
        let statuses: [JobStatus] = [.pending, .running, .completed, .failed, .paused, .cancelled]
        #expect(statuses.count == 6)
    }
}
