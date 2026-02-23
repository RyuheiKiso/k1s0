import Testing
@testable import K1s0Telemetry

@Suite("Telemetry Tests")
struct TelemetryTests {
    @Test("TelemetryConfig が正しく初期化されること")
    func testTelemetryConfigInit() {
        let config = TelemetryConfig(
            serviceName: "order-service",
            version: "1.0.0",
            tier: "service",
            environment: "dev"
        )
        #expect(config.serviceName == "order-service")
        #expect(config.sampleRate == 1.0)
        #expect(config.logLevel == .info)
    }

    @Test("Metrics がカウントをインクリメントすること")
    func testMetricsIncrement() async {
        let metrics = Metrics()
        await metrics.incrementHttpRequests(method: "GET", path: "/health", status: 200)
        await metrics.incrementHttpRequests(method: "GET", path: "/health", status: 200)
        let count = await metrics.httpRequestCount(method: "GET", path: "/health", status: 200)
        #expect(count == 2)
    }

    @Test("Prometheus 形式でエクスポートされること")
    func testPrometheusExport() async {
        let metrics = Metrics()
        await metrics.incrementHttpRequests(method: "POST", path: "/api/orders", status: 201)
        let exported = await metrics.exportPrometheus()
        #expect(exported.contains("http_requests_total"))
        #expect(exported.contains("POST"))
        #expect(exported.contains("/api/orders"))
    }

    @Test("LogLevel の rawValue が正しいこと")
    func testLogLevelRawValue() {
        #expect(LogLevel.debug.rawValue == "debug")
        #expect(LogLevel.info.rawValue == "info")
        #expect(LogLevel.warn.rawValue == "warn")
        #expect(LogLevel.error.rawValue == "error")
    }
}
