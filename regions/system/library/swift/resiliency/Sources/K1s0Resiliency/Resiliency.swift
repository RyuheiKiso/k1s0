import Foundation

// MARK: - Configuration

public struct RetryConfig: Sendable {
    public let maxAttempts: Int
    public let baseDelay: Duration
    public let maxDelay: Duration
    public let jitter: Bool

    public init(
        maxAttempts: Int = 3,
        baseDelay: Duration = .milliseconds(100),
        maxDelay: Duration = .seconds(5),
        jitter: Bool = true
    ) {
        self.maxAttempts = maxAttempts
        self.baseDelay = baseDelay
        self.maxDelay = maxDelay
        self.jitter = jitter
    }
}

public struct CircuitBreakerConfig: Sendable {
    public let failureThreshold: Int
    public let recoveryTimeout: Duration
    public let halfOpenMaxCalls: Int

    public init(
        failureThreshold: Int = 5,
        recoveryTimeout: Duration = .seconds(30),
        halfOpenMaxCalls: Int = 2
    ) {
        self.failureThreshold = failureThreshold
        self.recoveryTimeout = recoveryTimeout
        self.halfOpenMaxCalls = halfOpenMaxCalls
    }
}

public struct BulkheadConfig: Sendable {
    public let maxConcurrentCalls: Int
    public let maxWaitDuration: Duration

    public init(
        maxConcurrentCalls: Int = 20,
        maxWaitDuration: Duration = .milliseconds(500)
    ) {
        self.maxConcurrentCalls = maxConcurrentCalls
        self.maxWaitDuration = maxWaitDuration
    }
}

public struct ResiliencyPolicy: Sendable {
    public let retry: RetryConfig?
    public let circuitBreaker: CircuitBreakerConfig?
    public let bulkhead: BulkheadConfig?
    public let timeout: Duration?

    public init(
        retry: RetryConfig? = nil,
        circuitBreaker: CircuitBreakerConfig? = nil,
        bulkhead: BulkheadConfig? = nil,
        timeout: Duration? = nil
    ) {
        self.retry = retry
        self.circuitBreaker = circuitBreaker
        self.bulkhead = bulkhead
        self.timeout = timeout
    }
}

// MARK: - Errors

public enum ResiliencyError: Error, Sendable {
    case maxRetriesExceeded(attempts: Int, lastError: any Error & Sendable)
    case circuitBreakerOpen(remainingDuration: Duration)
    case bulkheadFull(maxConcurrent: Int)
    case timeout(after: Duration)
}

// MARK: - Circuit State

private enum CircuitState: Sendable {
    case closed
    case open
    case halfOpen
}

// MARK: - Decorator

public actor ResiliencyDecorator {
    private let policy: ResiliencyPolicy
    private var bulkheadCurrent: Int = 0
    private var cbState: CircuitState = .closed
    private var cbFailureCount: Int = 0
    private var cbSuccessCount: Int = 0
    private var cbLastFailureTime: ContinuousClock.Instant?

    public init(policy: ResiliencyPolicy) {
        self.policy = policy
    }

    public func execute<T: Sendable>(
        _ operation: @Sendable () async throws -> T
    ) async throws -> T {
        try checkCircuitBreaker()

        if policy.bulkhead != nil {
            try acquireBulkhead()
        }

        do {
            let result = try await executeWithRetry(operation)
            if policy.bulkhead != nil {
                releaseBulkhead()
            }
            return result
        } catch {
            if policy.bulkhead != nil {
                releaseBulkhead()
            }
            throw error
        }
    }

    private func executeWithRetry<T: Sendable>(
        _ operation: @Sendable () async throws -> T
    ) async throws -> T {
        let maxAttempts = policy.retry?.maxAttempts ?? 1
        var lastError: (any Error & Sendable)?

        for attempt in 0..<maxAttempts {
            do {
                let result = try await executeWithTimeout(operation)
                recordSuccess()
                return result
            } catch let error as ResiliencyError {
                throw error
            } catch {
                recordFailure()
                lastError = error
                try checkCircuitBreaker()

                if attempt + 1 < maxAttempts, let retryConfig = policy.retry {
                    let delay = calculateBackoff(
                        attempt: attempt,
                        baseDelay: retryConfig.baseDelay,
                        maxDelay: retryConfig.maxDelay
                    )
                    try await Task.sleep(for: delay)
                }
            }
        }

        throw ResiliencyError.maxRetriesExceeded(
            attempts: maxAttempts,
            lastError: lastError ?? CancellationError()
        )
    }

    private func executeWithTimeout<T: Sendable>(
        _ operation: @Sendable () async throws -> T
    ) async throws -> T {
        guard let timeoutDuration = policy.timeout else {
            return try await operation()
        }

        return try await withThrowingTaskGroup(of: T.self) { group in
            group.addTask {
                try await operation()
            }
            group.addTask {
                try await Task.sleep(for: timeoutDuration)
                throw ResiliencyError.timeout(after: timeoutDuration)
            }

            guard let result = try await group.next() else {
                throw ResiliencyError.timeout(after: timeoutDuration)
            }
            group.cancelAll()
            return result
        }
    }

    private func checkCircuitBreaker() throws {
        guard let cbConfig = policy.circuitBreaker else { return }

        switch cbState {
        case .closed:
            return
        case .open:
            if let lastFailure = cbLastFailureTime {
                let elapsed = ContinuousClock.now - lastFailure
                if elapsed >= cbConfig.recoveryTimeout {
                    cbState = .halfOpen
                    cbSuccessCount = 0
                    return
                }
                let remaining = cbConfig.recoveryTimeout - elapsed
                throw ResiliencyError.circuitBreakerOpen(remainingDuration: remaining)
            }
        case .halfOpen:
            return
        }
    }

    private func recordSuccess() {
        guard policy.circuitBreaker != nil else { return }

        switch cbState {
        case .halfOpen:
            cbSuccessCount += 1
            if cbSuccessCount >= policy.circuitBreaker!.halfOpenMaxCalls {
                cbState = .closed
                cbFailureCount = 0
            }
        case .closed:
            cbFailureCount = 0
        case .open:
            break
        }
    }

    private func recordFailure() {
        guard let cbConfig = policy.circuitBreaker else { return }

        cbFailureCount += 1
        if cbFailureCount >= cbConfig.failureThreshold {
            cbState = .open
            cbLastFailureTime = .now
        }
    }

    private func acquireBulkhead() throws {
        guard let bhConfig = policy.bulkhead else { return }

        if bulkheadCurrent < bhConfig.maxConcurrentCalls {
            bulkheadCurrent += 1
            return
        }

        throw ResiliencyError.bulkheadFull(maxConcurrent: bhConfig.maxConcurrentCalls)
    }

    private func releaseBulkhead() {
        bulkheadCurrent -= 1
    }

    private func calculateBackoff(attempt: Int, baseDelay: Duration, maxDelay: Duration) -> Duration {
        let multiplier = 1 << attempt
        let delayMs = baseDelay.components.seconds * 1000 + baseDelay.components.attoseconds / 1_000_000_000_000_000
        let calculatedMs = delayMs * Int64(multiplier)
        let maxMs = maxDelay.components.seconds * 1000 + maxDelay.components.attoseconds / 1_000_000_000_000_000
        let cappedMs = min(calculatedMs, maxMs)
        return .milliseconds(cappedMs)
    }
}

// MARK: - Convenience

public func withResiliency<T: Sendable>(
    policy: ResiliencyPolicy,
    operation: @Sendable () async throws -> T
) async throws -> T {
    let decorator = ResiliencyDecorator(policy: policy)
    return try await decorator.execute(operation)
}
