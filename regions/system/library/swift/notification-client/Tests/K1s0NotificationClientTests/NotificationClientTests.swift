import Testing
@testable import K1s0NotificationClient

@Suite("NotificationClient Tests")
struct NotificationClientTests {
    @Test("通知を送信できること")
    func testSend() async throws {
        let client = InMemoryNotificationClient()
        let req = NotificationRequest(channel: .email, recipient: "user@example.com", body: "Hello")
        let resp = try await client.send(req)
        #expect(resp.status == "sent")
        #expect(resp.id == req.id)
    }

    @Test("送信履歴が記録されること")
    func testSentHistory() async throws {
        let client = InMemoryNotificationClient()
        let req = NotificationRequest(channel: .sms, recipient: "+1234567890", body: "Code: 1234")
        _ = try await client.send(req)
        let sent = await client.sent()
        #expect(sent.count == 1)
        #expect(sent[0].channel == .sms)
    }

    @Test("全チャネルが使用できること")
    func testAllChannels() async throws {
        let client = InMemoryNotificationClient()
        let channels: [NotificationChannel] = [.email, .sms, .push, .webhook]
        for channel in channels {
            let req = NotificationRequest(channel: channel, recipient: "test", body: "test")
            let resp = try await client.send(req)
            #expect(resp.status == "sent")
        }
        let sent = await client.sent()
        #expect(sent.count == 4)
    }

    @Test("subjectがオプショナルであること")
    func testOptionalSubject() async throws {
        let withSubject = NotificationRequest(channel: .email, recipient: "user@example.com", body: "Hello", subject: "Greetings")
        #expect(withSubject.subject == "Greetings")

        let withoutSubject = NotificationRequest(channel: .push, recipient: "device-token", body: "Alert")
        #expect(withoutSubject.subject == nil)
    }

    @Test("NotificationRequestにIDが自動設定されること")
    func testAutoID() {
        let req = NotificationRequest(channel: .email, recipient: "test", body: "test")
        #expect(!req.id.isEmpty)
    }
}
