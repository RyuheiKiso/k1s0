import Testing
@testable import K1s0Idempotency
import Foundation

@Suite("Idempotency Tests")
struct IdempotencyTests {
    @Test("レコードを保存して取得できること")
    func testSaveAndGet() async throws {
        let store = InMemoryIdempotencyStore()
        let record = IdempotencyRecord(key: "req-001", status: .pending)
        try await store.save(record)

        let retrieved = try await store.get("req-001")
        #expect(retrieved != nil)
        #expect(retrieved?.key == "req-001")
        #expect(retrieved?.status == .pending)
    }

    @Test("存在しないキーを取得すると nil が返ること")
    func testGetNonexistentReturnsNil() async throws {
        let store = InMemoryIdempotencyStore()
        let result = try await store.get("missing-key")
        #expect(result == nil)
    }

    @Test("レコードを削除できること")
    func testDeleteRecord() async throws {
        let store = InMemoryIdempotencyStore()
        let record = IdempotencyRecord(key: "req-002", status: .completed)
        try await store.save(record)

        try await store.delete("req-002")
        let result = try await store.get("req-002")
        #expect(result == nil)
    }

    @Test("有効期限切れのレコードは取得できないこと")
    func testExpiryRecord() async throws {
        let store = InMemoryIdempotencyStore()
        let record = IdempotencyRecord(
            key: "req-003",
            status: .completed,
            expiresAt: Date().addingTimeInterval(-1.0)
        )
        try await store.save(record)

        let result = try await store.get("req-003")
        #expect(result == nil)
    }

    @Test("ステータス遷移が正しく保存されること")
    func testStatusTransitions() async throws {
        let store = InMemoryIdempotencyStore()
        var record = IdempotencyRecord(key: "req-004", status: .pending)
        try await store.save(record)

        record.status = .processing
        try await store.save(record)
        let processing = try await store.get("req-004")
        #expect(processing?.status == .processing)

        record.status = .completed
        record.response = "result".data(using: .utf8)
        try await store.save(record)
        let completed = try await store.get("req-004")
        #expect(completed?.status == .completed)
        #expect(completed?.response != nil)
    }
}
