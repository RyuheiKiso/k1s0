import Foundation

public enum HealthStatus: String, Sendable {
    case healthy
    case degraded
    case unhealthy
}

public struct CheckResult: Sendable {
    public let status: HealthStatus
    public let message: String?

    public init(status: HealthStatus, message: String? = nil) {
        self.status = status
        self.message = message
    }
}

public struct HealthResponse: Sendable {
    public let status: HealthStatus
    public let checks: [String: CheckResult]
    public let timestamp: Date

    public init(status: HealthStatus, checks: [String: CheckResult], timestamp: Date) {
        self.status = status
        self.checks = checks
        self.timestamp = timestamp
    }
}

public protocol HealthCheck: Sendable {
    var name: String { get }
    func check() async throws
}
