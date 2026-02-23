import Testing
@testable import K1s0Health

struct AlwaysHealthy: HealthCheck {
    let name = "always-healthy"
    func check() async throws {}
}

struct AlwaysUnhealthy: HealthCheck {
    let name = "always-unhealthy"
    func check() async throws {
        throw HealthTestError.failed
    }
}

enum HealthTestError: Error {
    case failed
}

@Suite("Health Tests")
struct HealthTests {
    @Test("全てのチェックが正常な場合healthyになること")
    func testAllHealthy() async {
        let checker = HealthChecker()
        await checker.add(AlwaysHealthy())
        let response = await checker.runAll()
        #expect(response.status == .healthy)
        #expect(response.checks["always-healthy"]?.status == .healthy)
    }

    @Test("異常なチェックがある場合unhealthyになること")
    func testUnhealthy() async {
        let checker = HealthChecker()
        await checker.add(AlwaysUnhealthy())
        let response = await checker.runAll()
        #expect(response.status == .unhealthy)
        #expect(response.checks["always-unhealthy"]?.status == .unhealthy)
    }

    @Test("混合状態の場合unhealthyになること")
    func testMixed() async {
        let checker = HealthChecker()
        await checker.add(AlwaysHealthy())
        await checker.add(AlwaysUnhealthy())
        let response = await checker.runAll()
        #expect(response.status == .unhealthy)
        #expect(response.checks.count == 2)
    }

    @Test("チェックが空の場合healthyになること")
    func testEmpty() async {
        let checker = HealthChecker()
        let response = await checker.runAll()
        #expect(response.status == .healthy)
        #expect(response.checks.isEmpty)
    }

    @Test("HealthResponseにタイムスタンプが含まれること")
    func testTimestamp() async {
        let checker = HealthChecker()
        let response = await checker.runAll()
        #expect(response.timestamp.timeIntervalSinceNow < 1.0)
    }
}
