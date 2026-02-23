import Testing
@testable import K1s0Tracing

@Suite("Tracing Tests")
struct TracingTests {
    @Test("TraceContextをtraceparentに変換できること")
    func testToTraceparent() {
        let ctx = TraceContext(
            traceId: "0af7651916cd43dd8448eb211c80319c",
            parentId: "b7ad6b7169203331",
            flags: 1
        )
        let header = ctx.toTraceparent()
        #expect(header == "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
    }

    @Test("traceparentからTraceContextをパースできること")
    func testFromTraceparent() {
        let ctx = TraceContext.fromTraceparent("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
        #expect(ctx != nil)
        #expect(ctx?.traceId == "0af7651916cd43dd8448eb211c80319c")
        #expect(ctx?.parentId == "b7ad6b7169203331")
        #expect(ctx?.flags == 1)
    }

    @Test("不正なtraceparentでnilを返すこと")
    func testInvalidTraceparent() {
        #expect(TraceContext.fromTraceparent("invalid") == nil)
        #expect(TraceContext.fromTraceparent("01-abc-def-00") == nil)
        #expect(TraceContext.fromTraceparent("") == nil)
    }

    @Test("Baggageのset/getが動作すること")
    func testBaggage() {
        var baggage = Baggage()
        baggage.set("key1", "value1")
        baggage.set("key2", "value2")
        #expect(baggage.get("key1") == "value1")
        #expect(baggage.get("key2") == "value2")
        #expect(baggage.get("key3") == nil)
    }

    @Test("Baggageをヘッダーに変換できること")
    func testBaggageToHeader() {
        var baggage = Baggage()
        baggage.set("key1", "value1")
        let header = baggage.toHeader()
        #expect(header.contains("key1=value1"))
    }

    @Test("ヘッダーからBaggageをパースできること")
    func testBaggageFromHeader() {
        let baggage = Baggage.fromHeader("key1=value1,key2=value2")
        #expect(baggage.get("key1") == "value1")
        #expect(baggage.get("key2") == "value2")
    }

    @Test("空文字列からBaggageをパースできること")
    func testBaggageFromEmptyHeader() {
        let baggage = Baggage.fromHeader("")
        #expect(baggage.isEmpty)
    }

    @Test("コンテキストをヘッダーに注入できること")
    func testInjectContext() {
        var headers: [String: String] = [:]
        let ctx = TraceContext(
            traceId: "0af7651916cd43dd8448eb211c80319c",
            parentId: "b7ad6b7169203331",
            flags: 1
        )
        var baggage = Baggage()
        baggage.set("userId", "123")

        injectContext(into: &headers, context: ctx, baggage: baggage)
        #expect(headers["traceparent"] == "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
        #expect(headers["baggage"]?.contains("userId=123") == true)
    }

    @Test("ヘッダーからコンテキストを抽出できること")
    func testExtractContext() {
        let headers = [
            "traceparent": "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
            "baggage": "userId=123"
        ]
        let (ctx, baggage) = extractContext(from: headers)
        #expect(ctx?.traceId == "0af7651916cd43dd8448eb211c80319c")
        #expect(baggage.get("userId") == "123")
    }

    @Test("ヘッダーなしでコンテキスト抽出するとnilを返すこと")
    func testExtractContextEmpty() {
        let (ctx, baggage) = extractContext(from: [:])
        #expect(ctx == nil)
        #expect(baggage.isEmpty)
    }

    @Test("baggage無しで注入した場合baggageヘッダーが追加されないこと")
    func testInjectWithoutBaggage() {
        var headers: [String: String] = [:]
        let ctx = TraceContext(
            traceId: "0af7651916cd43dd8448eb211c80319c",
            parentId: "b7ad6b7169203331"
        )
        injectContext(into: &headers, context: ctx)
        #expect(headers["traceparent"] != nil)
        #expect(headers["baggage"] == nil)
    }
}
