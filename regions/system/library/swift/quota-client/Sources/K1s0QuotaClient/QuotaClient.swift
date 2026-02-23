import Foundation

public enum QuotaPeriod: Sendable, Codable, Equatable {
    case hourly
    case daily
    case monthly
    case custom(seconds: Int)
}

public struct QuotaStatus: Sendable, Codable, Equatable {
    public let allowed: Bool
    public let remaining: UInt64
    public let limit: UInt64
    public let resetAt: Date

    public init(allowed: Bool, remaining: UInt64, limit: UInt64, resetAt: Date) {
        self.allowed = allowed
        self.remaining = remaining
        self.limit = limit
        self.resetAt = resetAt
    }
}

public struct QuotaUsage: Sendable, Codable, Equatable {
    public let quotaId: String
    public let used: UInt64
    public let limit: UInt64
    public let period: QuotaPeriod
    public let resetAt: Date

    public init(quotaId: String, used: UInt64, limit: UInt64, period: QuotaPeriod, resetAt: Date) {
        self.quotaId = quotaId
        self.used = used
        self.limit = limit
        self.period = period
        self.resetAt = resetAt
    }
}

public struct QuotaPolicy: Sendable, Codable, Equatable {
    public let quotaId: String
    public let limit: UInt64
    public let period: QuotaPeriod
    public let resetStrategy: String

    public init(quotaId: String, limit: UInt64, period: QuotaPeriod, resetStrategy: String) {
        self.quotaId = quotaId
        self.limit = limit
        self.period = period
        self.resetStrategy = resetStrategy
    }
}

public struct QuotaClientConfig: Sendable {
    public let serverUrl: URL
    public let timeout: Duration
    public let policyCacheTtl: Duration

    public init(
        serverUrl: URL,
        timeout: Duration = .seconds(5),
        policyCacheTtl: Duration = .seconds(60)
    ) {
        self.serverUrl = serverUrl
        self.timeout = timeout
        self.policyCacheTtl = policyCacheTtl
    }
}

public enum QuotaClientError: Error, Sendable {
    case connectionFailed(underlying: any Error)
    case quotaExceeded(quotaId: String, remaining: UInt64)
    case notFound(quotaId: String)
    case invalidResponse
}

public protocol QuotaClientProtocol: Sendable {
    func check(quotaId: String, amount: UInt64) async throws -> QuotaStatus
    func increment(quotaId: String, amount: UInt64) async throws -> QuotaUsage
    func getUsage(quotaId: String) async throws -> QuotaUsage
    func getPolicy(quotaId: String) async throws -> QuotaPolicy
}

public actor InMemoryQuotaClient: QuotaClientProtocol {
    private var usages: [String: UsageEntry] = [:]
    private var policies: [String: QuotaPolicy] = [:]

    public init() {}

    public func setPolicy(_ quotaId: String, policy: QuotaPolicy) {
        policies[quotaId] = policy
    }

    private func getOrCreateUsage(_ quotaId: String) -> UsageEntry {
        if let entry = usages[quotaId] {
            return entry
        }
        let policy = policies[quotaId]
        let entry = UsageEntry(
            quotaId: quotaId,
            used: 0,
            limit: policy?.limit ?? 1000,
            period: policy?.period ?? .daily,
            resetAt: Date().addingTimeInterval(86400)
        )
        usages[quotaId] = entry
        return entry
    }

    public func check(quotaId: String, amount: UInt64) async throws -> QuotaStatus {
        let usage = getOrCreateUsage(quotaId)
        let remaining = usage.limit - usage.used
        return QuotaStatus(
            allowed: amount <= remaining,
            remaining: remaining,
            limit: usage.limit,
            resetAt: usage.resetAt
        )
    }

    public func increment(quotaId: String, amount: UInt64) async throws -> QuotaUsage {
        let usage = getOrCreateUsage(quotaId)
        usage.used += amount
        usages[quotaId] = usage
        return QuotaUsage(
            quotaId: usage.quotaId,
            used: usage.used,
            limit: usage.limit,
            period: usage.period,
            resetAt: usage.resetAt
        )
    }

    public func getUsage(quotaId: String) async throws -> QuotaUsage {
        let usage = getOrCreateUsage(quotaId)
        return QuotaUsage(
            quotaId: usage.quotaId,
            used: usage.used,
            limit: usage.limit,
            period: usage.period,
            resetAt: usage.resetAt
        )
    }

    public func getPolicy(quotaId: String) async throws -> QuotaPolicy {
        if let policy = policies[quotaId] {
            return policy
        }
        return QuotaPolicy(
            quotaId: quotaId,
            limit: 1000,
            period: .daily,
            resetStrategy: "fixed"
        )
    }
}

final class UsageEntry: @unchecked Sendable {
    let quotaId: String
    var used: UInt64
    let limit: UInt64
    let period: QuotaPeriod
    let resetAt: Date

    init(quotaId: String, used: UInt64, limit: UInt64, period: QuotaPeriod, resetAt: Date) {
        self.quotaId = quotaId
        self.used = used
        self.limit = limit
        self.period = period
        self.resetAt = resetAt
    }
}
