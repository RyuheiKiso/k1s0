import Foundation

public enum JobStatus: String, Sendable {
    case pending
    case running
    case completed
    case failed
    case paused
    case cancelled
}

public enum Schedule: Sendable {
    case cron(String)
    case oneShot(Date)
    case interval(Duration)
}

public struct JobRequest: Sendable {
    public let name: String
    public let schedule: Schedule
    public let payload: [String: String]
    public let maxRetries: UInt32
    public let timeoutSecs: UInt64

    public init(
        name: String,
        schedule: Schedule,
        payload: [String: String] = [:],
        maxRetries: UInt32 = 3,
        timeoutSecs: UInt64 = 60
    ) {
        self.name = name
        self.schedule = schedule
        self.payload = payload
        self.maxRetries = maxRetries
        self.timeoutSecs = timeoutSecs
    }
}

public struct Job: Sendable {
    public let id: String
    public let name: String
    public let schedule: Schedule
    public let status: JobStatus
    public let payload: [String: String]
    public let maxRetries: UInt32
    public let timeoutSecs: UInt64
    public let createdAt: Date
    public let nextRunAt: Date?

    public init(
        id: String,
        name: String,
        schedule: Schedule,
        status: JobStatus,
        payload: [String: String] = [:],
        maxRetries: UInt32 = 3,
        timeoutSecs: UInt64 = 60,
        createdAt: Date = Date(),
        nextRunAt: Date? = nil
    ) {
        self.id = id
        self.name = name
        self.schedule = schedule
        self.status = status
        self.payload = payload
        self.maxRetries = maxRetries
        self.timeoutSecs = timeoutSecs
        self.createdAt = createdAt
        self.nextRunAt = nextRunAt
    }
}

public struct JobFilter: Sendable {
    public let status: JobStatus?
    public let namePrefix: String?

    public init(status: JobStatus? = nil, namePrefix: String? = nil) {
        self.status = status
        self.namePrefix = namePrefix
    }
}

public struct JobExecution: Sendable {
    public let id: String
    public let jobId: String
    public let startedAt: Date
    public let finishedAt: Date?
    public let result: String
    public let error: String?

    public init(
        id: String,
        jobId: String,
        startedAt: Date,
        finishedAt: Date? = nil,
        result: String,
        error: String? = nil
    ) {
        self.id = id
        self.jobId = jobId
        self.startedAt = startedAt
        self.finishedAt = finishedAt
        self.result = result
        self.error = error
    }
}

public struct JobCompletedEvent: Sendable {
    public let jobId: String
    public let executionId: String
    public let completedAt: Date
    public let result: String

    public init(jobId: String, executionId: String, completedAt: Date, result: String) {
        self.jobId = jobId
        self.executionId = executionId
        self.completedAt = completedAt
        self.result = result
    }
}

public enum SchedulerError: Error, Sendable {
    case jobNotFound(jobId: String)
    case invalidSchedule(reason: String)
    case serverError(message: String)
    case timeout
}

public protocol SchedulerClient: Sendable {
    func createJob(_ req: JobRequest) async throws -> Job
    func cancelJob(jobId: String) async throws
    func pauseJob(jobId: String) async throws
    func resumeJob(jobId: String) async throws
    func getJob(jobId: String) async throws -> Job
    func listJobs(filter: JobFilter?) async throws -> [Job]
    func getExecutions(jobId: String) async throws -> [JobExecution]
}

public actor InMemorySchedulerClient: SchedulerClient {
    private var jobs: [String: Job] = [:]
    private var seq = 0

    public init() {}

    public func allJobs() -> [String: Job] {
        jobs
    }

    public func createJob(_ req: JobRequest) async throws -> Job {
        seq += 1
        let id = String(format: "job-%03d", seq)
        let job = Job(
            id: id,
            name: req.name,
            schedule: req.schedule,
            status: .pending,
            payload: req.payload,
            maxRetries: req.maxRetries,
            timeoutSecs: req.timeoutSecs
        )
        jobs[id] = job
        return job
    }

    public func cancelJob(jobId: String) async throws {
        guard let job = jobs[jobId] else {
            throw SchedulerError.jobNotFound(jobId: jobId)
        }
        jobs[jobId] = Job(
            id: job.id, name: job.name, schedule: job.schedule,
            status: .cancelled, payload: job.payload,
            maxRetries: job.maxRetries, timeoutSecs: job.timeoutSecs,
            createdAt: job.createdAt, nextRunAt: job.nextRunAt
        )
    }

    public func pauseJob(jobId: String) async throws {
        guard let job = jobs[jobId] else {
            throw SchedulerError.jobNotFound(jobId: jobId)
        }
        jobs[jobId] = Job(
            id: job.id, name: job.name, schedule: job.schedule,
            status: .paused, payload: job.payload,
            maxRetries: job.maxRetries, timeoutSecs: job.timeoutSecs,
            createdAt: job.createdAt, nextRunAt: job.nextRunAt
        )
    }

    public func resumeJob(jobId: String) async throws {
        guard let job = jobs[jobId] else {
            throw SchedulerError.jobNotFound(jobId: jobId)
        }
        jobs[jobId] = Job(
            id: job.id, name: job.name, schedule: job.schedule,
            status: .pending, payload: job.payload,
            maxRetries: job.maxRetries, timeoutSecs: job.timeoutSecs,
            createdAt: job.createdAt, nextRunAt: job.nextRunAt
        )
    }

    public func getJob(jobId: String) async throws -> Job {
        guard let job = jobs[jobId] else {
            throw SchedulerError.jobNotFound(jobId: jobId)
        }
        return job
    }

    public func listJobs(filter: JobFilter?) async throws -> [Job] {
        var result = Array(jobs.values)
        if let status = filter?.status {
            result = result.filter { $0.status == status }
        }
        if let prefix = filter?.namePrefix, !prefix.isEmpty {
            result = result.filter { $0.name.hasPrefix(prefix) }
        }
        return result
    }

    public func getExecutions(jobId: String) async throws -> [JobExecution] {
        []
    }
}
