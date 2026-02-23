import Foundation

public enum CircuitState: Sendable {
    case closed
    case open
    case halfOpen
}

public struct CircuitBreakerConfig: Sendable {
    public let failureThreshold: Int
    public let successThreshold: Int
    public let timeout: TimeInterval

    public init(failureThreshold: Int, successThreshold: Int, timeout: TimeInterval) {
        self.failureThreshold = failureThreshold
        self.successThreshold = successThreshold
        self.timeout = timeout
    }
}

public enum CircuitBreakerError: Error, Sendable {
    case open
}

public actor CircuitBreaker {
    private let config: CircuitBreakerConfig
    private var state: CircuitState = .closed
    private var failureCount = 0
    private var successCount = 0
    private var openedAt: Date?

    public init(config: CircuitBreakerConfig) {
        self.config = config
    }

    public func currentState() -> CircuitState {
        state
    }

    public func isOpen() -> Bool {
        state == .open
    }

    public func recordSuccess() {
        switch state {
        case .closed:
            failureCount = 0
        case .halfOpen:
            successCount += 1
            if successCount >= config.successThreshold {
                state = .closed
                failureCount = 0
                successCount = 0
            }
        case .open:
            break
        }
    }

    public func recordFailure() {
        switch state {
        case .closed:
            failureCount += 1
            if failureCount >= config.failureThreshold {
                state = .open
                openedAt = Date()
                failureCount = 0
            }
        case .halfOpen:
            state = .open
            openedAt = Date()
            successCount = 0
        case .open:
            break
        }
    }

    public func call<T: Sendable>(_ fn: @Sendable () async throws -> T) async throws -> T {
        if state == .open {
            if let openedAt, Date().timeIntervalSince(openedAt) >= config.timeout {
                state = .halfOpen
                successCount = 0
            } else {
                throw CircuitBreakerError.open
            }
        }

        do {
            let result = try await fn()
            recordSuccess()
            return result
        } catch {
            recordFailure()
            throw error
        }
    }
}
