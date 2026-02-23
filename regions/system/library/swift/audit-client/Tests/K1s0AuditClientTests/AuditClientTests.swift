import Testing
@testable import K1s0AuditClient

@Suite("AuditClient Tests")
struct AuditClientTests {
    @Test("イベントを記録できること")
    func testRecord() async throws {
        let client = BufferedAuditClient()
        let event = AuditEvent(tenantId: "t1", actorId: "u1", action: "create", resourceType: "user", resourceId: "r1")
        try await client.record(event)
        let flushed = try await client.flush()
        #expect(flushed.count == 1)
        #expect(flushed[0].action == "create")
    }

    @Test("flushでバッファがクリアされること")
    func testFlushClearsBuffer() async throws {
        let client = BufferedAuditClient()
        let event = AuditEvent(tenantId: "t1", actorId: "u1", action: "delete", resourceType: "item", resourceId: "r2")
        try await client.record(event)
        _ = try await client.flush()
        let empty = try await client.flush()
        #expect(empty.isEmpty)
    }

    @Test("複数イベントを記録できること")
    func testMultipleRecords() async throws {
        let client = BufferedAuditClient()
        for i in 0..<5 {
            let event = AuditEvent(tenantId: "t1", actorId: "u1", action: "update", resourceType: "doc", resourceId: "r\(i)")
            try await client.record(event)
        }
        let flushed = try await client.flush()
        #expect(flushed.count == 5)
    }

    @Test("AuditEventにIDとタイムスタンプが自動設定されること")
    func testEventAutoFields() {
        let event = AuditEvent(tenantId: "t1", actorId: "u1", action: "read", resourceType: "file", resourceId: "f1")
        #expect(!event.id.isEmpty)
        #expect(event.timestamp.timeIntervalSinceNow < 1.0)
    }

    @Test("空バッファのflushが空配列を返すこと")
    func testFlushEmpty() async throws {
        let client = BufferedAuditClient()
        let result = try await client.flush()
        #expect(result.isEmpty)
    }
}
