import Foundation

public enum NotificationChannel: String, Sendable {
    case email
    case sms
    case push
    case webhook
}

public struct NotificationRequest: Sendable {
    public let id: String
    public let channel: NotificationChannel
    public let recipient: String
    public let subject: String?
    public let body: String

    public init(channel: NotificationChannel, recipient: String, body: String, subject: String? = nil) {
        self.id = UUID().uuidString
        self.channel = channel
        self.recipient = recipient
        self.subject = subject
        self.body = body
    }
}

public struct NotificationResponse: Sendable {
    public let id: String
    public let status: String

    public init(id: String, status: String) {
        self.id = id
        self.status = status
    }
}

public protocol NotificationClient: Sendable {
    func send(_ request: NotificationRequest) async throws -> NotificationResponse
}

public actor InMemoryNotificationClient: NotificationClient {
    private var sentRequests: [NotificationRequest] = []

    public init() {}

    public func sent() -> [NotificationRequest] {
        sentRequests
    }

    public func send(_ request: NotificationRequest) async throws -> NotificationResponse {
        sentRequests.append(request)
        return NotificationResponse(id: request.id, status: "sent")
    }
}
