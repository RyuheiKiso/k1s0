import Foundation

public actor HealthChecker {
    private var checks: [any HealthCheck] = []

    public init() {}

    public func add(_ check: some HealthCheck) {
        checks.append(check)
    }

    public func runAll() async -> HealthResponse {
        var results: [String: CheckResult] = [:]
        var overall = HealthStatus.healthy
        for c in checks {
            do {
                try await c.check()
                results[c.name] = CheckResult(status: .healthy)
            } catch {
                results[c.name] = CheckResult(status: .unhealthy, message: error.localizedDescription)
                overall = .unhealthy
            }
        }
        return HealthResponse(status: overall, checks: results, timestamp: Date())
    }
}
