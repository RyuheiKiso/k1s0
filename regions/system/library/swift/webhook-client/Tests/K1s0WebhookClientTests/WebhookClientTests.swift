import Testing
import Foundation
@testable import K1s0WebhookClient

@Suite("WebhookClient Tests")
struct WebhookClientTests {
    @Test("署名の生成と検証が正しく動作すること")
    func testSignatureRoundTrip() {
        let secret = "my-secret"
        let body = Data("payload".utf8)
        let signature = generateSignature(secret: secret, body: body)
        #expect(verifySignature(secret: secret, body: body, signature: signature))
    }

    @Test("異なるシークレットで署名検証が失敗すること")
    func testSignatureMismatch() {
        let body = Data("payload".utf8)
        let signature = generateSignature(secret: "secret1", body: body)
        #expect(!verifySignature(secret: "secret2", body: body, signature: signature))
    }

    @Test("InMemoryWebhookClientが送信を記録すること")
    func testSendRecordsWebhook() async throws {
        let client = InMemoryWebhookClient()
        let payload = WebhookPayload(eventType: "user.created", timestamp: "2026-01-01T00:00:00Z", data: ["id": "123"])
        let status = try await client.send(url: "https://example.com/webhook", payload: payload)
        #expect(status == 200)
        let sent = await client.sent()
        #expect(sent.count == 1)
        #expect(sent[0].payload.eventType == "user.created")
    }

    @Test("無効なURLがエラーになること")
    func testInvalidURL() async {
        let client = InMemoryWebhookClient()
        let payload = WebhookPayload(eventType: "test", timestamp: "now", data: [:])
        do {
            _ = try await client.send(url: "", payload: payload)
            #expect(Bool(false), "Should have thrown")
        } catch is WebhookError {
            // expected
        } catch {
            #expect(Bool(false), "Unexpected error")
        }
    }

    @Test("同じ入力に対して署名が決定的であること")
    func testSignatureDeterministic() {
        let secret = "key"
        let body = Data("data".utf8)
        let sig1 = generateSignature(secret: secret, body: body)
        let sig2 = generateSignature(secret: secret, body: body)
        #expect(sig1 == sig2)
    }
}
