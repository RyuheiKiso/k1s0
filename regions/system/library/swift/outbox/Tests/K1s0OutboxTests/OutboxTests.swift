import Foundation
import Testing
@testable import K1s0Outbox

@Suite("Outbox Tests")
struct OutboxTests {
    @Test("メッセージが pending で初期化されること")
    func testMessageInitialStatus() {
        let msg = OutboxMessage(topic: "test.topic", partitionKey: "key-1", payload: Data())
        #expect(msg.status == .pending)
        #expect(msg.retryCount == 0)
        #expect(msg.isProcessable)
    }

    @Test("delivered に遷移できること")
    func testMarkDelivered() {
        var msg = OutboxMessage(topic: "test.topic", partitionKey: "key-1", payload: Data())
        msg.markDelivered()
        #expect(msg.status == .delivered)
    }

    @Test("失敗時にリトライカウントが増加すること")
    func testMarkFailed() {
        var msg = OutboxMessage(topic: "test.topic", partitionKey: "key-1", payload: Data(), maxRetries: 3)
        msg.markFailed(error: "connection refused")
        #expect(msg.retryCount == 1)
        #expect(msg.status == .failed)
        #expect(msg.lastError == "connection refused")
    }

    @Test("maxRetries 超過で deadLetter になること")
    func testMaxRetriesExceeded() {
        var msg = OutboxMessage(topic: "test.topic", partitionKey: "key-1", payload: Data(), maxRetries: 2)
        msg.markFailed(error: "error 1")
        msg.markFailed(error: "error 2")
        #expect(msg.status == .deadLetter)
    }

    @Test("OutboxError の説明が含まれること")
    func testOutboxErrorDescription() {
        let error = OutboxError.storeError("db connection failed")
        #expect(error.description.contains("STORE_ERROR"))
    }
}
