import Foundation

public struct RateLimitStatus: Sendable {
    public let allowed: Bool
    public let remaining: UInt32
    public let resetAt: Date
    public let retryAfterSecs: UInt64?

    public init(allowed: Bool, remaining: UInt32, resetAt: Date, retryAfterSecs: UInt64? = nil) {
        self.allowed = allowed
        self.remaining = remaining
        self.resetAt = resetAt
        self.retryAfterSecs = retryAfterSecs
    }
}

public struct RateLimitResult: Sendable {
    public let remaining: UInt32
    public let resetAt: Date

    public init(remaining: UInt32, resetAt: Date) {
        self.remaining = remaining
        self.resetAt = resetAt
    }
}

public struct RateLimitPolicy: Sendable {
    public let key: String
    public let limit: UInt32
    public let windowSecs: UInt64
    public let algorithm: String

    public init(key: String, limit: UInt32, windowSecs: UInt64, algorithm: String) {
        self.key = key
        self.limit = limit
        self.windowSecs = windowSecs
        self.algorithm = algorithm
    }
}

public enum RateLimitError: Error, Sendable {
    case limitExceeded(retryAfterSecs: UInt64)
    case keyNotFound(key: String)
    case serverError(message: String)
    case timeout
}

public protocol RateLimitClient: Sendable {
    func check(key: String, cost: UInt32) async throws -> RateLimitStatus
    func consume(key: String, cost: UInt32) async throws -> RateLimitResult
    func getLimit(key: String) async throws -> RateLimitPolicy
}

public actor InMemoryRateLimitClient: RateLimitClient {
    private var counters: [String: UInt32] = [:]
    private var policies: [String: RateLimitPolicy] = [:]

    private static let defaultPolicy = RateLimitPolicy(
        key: "default", limit: 100, windowSecs: 3600, algorithm: "token_bucket"
    )

    public init() {}

    public func setPolicy(key: String, policy: RateLimitPolicy) {
        policies[key] = policy
    }

    private func getPolicy(key: String) -> RateLimitPolicy {
        policies[key] ?? InMemoryRateLimitClient.defaultPolicy
    }

    public func check(key: String, cost: UInt32) async throws -> RateLimitStatus {
        let policy = getPolicy(key: key)
        let used = counters[key] ?? 0
        let resetAt = Date().addingTimeInterval(Double(policy.windowSecs))

        if used + cost > policy.limit {
            return RateLimitStatus(
                allowed: false, remaining: 0, resetAt: resetAt,
                retryAfterSecs: policy.windowSecs
            )
        }

        return RateLimitStatus(
            allowed: true, remaining: policy.limit - used - cost, resetAt: resetAt
        )
    }

    public func consume(key: String, cost: UInt32) async throws -> RateLimitResult {
        let policy = getPolicy(key: key)
        let used = counters[key] ?? 0

        if used + cost > policy.limit {
            throw RateLimitError.limitExceeded(retryAfterSecs: policy.windowSecs)
        }

        counters[key] = used + cost
        let remaining = policy.limit - (used + cost)
        let resetAt = Date().addingTimeInterval(Double(policy.windowSecs))

        return RateLimitResult(remaining: remaining, resetAt: resetAt)
    }

    public func getLimit(key: String) async throws -> RateLimitPolicy {
        getPolicy(key: key)
    }

    public func getUsedCount(key: String) -> UInt32 {
        counters[key] ?? 0
    }
}
