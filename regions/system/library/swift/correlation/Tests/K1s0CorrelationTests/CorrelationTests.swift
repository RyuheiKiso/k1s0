import Testing
@testable import K1s0Correlation

@Suite("Correlation Tests")
struct CorrelationTests {
    @Test("CorrelationId が生成されること")
    func testCorrelationIdGeneration() {
        let id1 = CorrelationId()
        let id2 = CorrelationId()
        #expect(id1 != id2)
        #expect(!id1.asString.isEmpty)
    }

    @Test("TraceId が32文字16進数であること")
    func testTraceIdFormat() {
        let id = TraceId()
        #expect(id.asString.count == 32)
        #expect(id.asString.allSatisfy { $0.isHexDigit })
    }

    @Test("不正なTraceIdはnilになること")
    func testInvalidTraceId() {
        #expect(TraceId.from(string: "short") == nil)
        #expect(TraceId.from(string: "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ") == nil)
    }

    @Test("ヘッダーの変換が正しいこと")
    func testHeaderConversion() {
        var ctx = CorrelationContext()
        let traceId = TraceId()
        ctx = ctx.withTraceId(traceId)
        let headers = CorrelationHeaders.toHeaders(ctx)
        let restored = CorrelationHeaders.fromHeaders(headers)
        #expect(restored.correlationId == ctx.correlationId)
        #expect(restored.traceId == ctx.traceId)
    }

    @Test("大文字小文字混在ヘッダーが正しく処理されること")
    func testCaseInsensitiveHeaders() {
        let id = CorrelationId(string: "test-id")
        let headers: [(String, String)] = [
            ("X-Correlation-Id", "test-id"),
        ]
        let ctx = CorrelationHeaders.fromHeaders(headers)
        #expect(ctx.correlationId == id)
    }
}
