import Testing
@testable import K1s0Health

@Suite("HttpHealthCheck Tests")
struct HttpHealthCheckTests {
    @Test("デフォルト名が'http'であること")
    func testDefaultName() {
        let check = HttpHealthCheck(url: "http://example.com/healthz")
        #expect(check.name == "http")
    }

    @Test("カスタム名を設定できること")
    func testCustomName() {
        let check = HttpHealthCheck(url: "http://example.com/healthz", name: "upstream")
        #expect(check.name == "upstream")
    }

    @Test("HealthCheckerと統合できること")
    func testIntegrationWithChecker() async {
        let checker = HealthChecker()
        let check = HttpHealthCheck(url: "http://example.com/healthz", name: "upstream")
        await checker.add(check)
        // HttpHealthCheck が HealthCheck プロトコルに準拠していることを確認
        #expect(check.name == "upstream")
    }
}
