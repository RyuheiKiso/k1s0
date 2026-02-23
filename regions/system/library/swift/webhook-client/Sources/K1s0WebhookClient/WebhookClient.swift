import Foundation
import Crypto

public struct WebhookPayload: Sendable {
    public let eventType: String
    public let timestamp: String
    public let data: [String: String]

    public init(eventType: String, timestamp: String, data: [String: String]) {
        self.eventType = eventType
        self.timestamp = timestamp
        self.data = data
    }
}

public enum WebhookError: Error, Sendable {
    case invalidURL(String)
    case sendFailed(String)
}

public func generateSignature(secret: String, body: Data) -> String {
    let key = SymmetricKey(data: Data(secret.utf8))
    let mac = HMAC<SHA256>.authenticationCode(for: body, using: key)
    return Data(mac).map { String(format: "%02x", $0) }.joined()
}

public func verifySignature(secret: String, body: Data, signature: String) -> Bool {
    generateSignature(secret: secret, body: body) == signature
}

public protocol WebhookClient: Sendable {
    func send(url: String, payload: WebhookPayload) async throws -> Int
}

public actor InMemoryWebhookClient: WebhookClient {
    public struct SentWebhook: Sendable {
        public let url: String
        public let payload: WebhookPayload
    }

    private var sentWebhooks: [SentWebhook] = []

    public init() {}

    public func sent() -> [SentWebhook] {
        sentWebhooks
    }

    public func send(url: String, payload: WebhookPayload) async throws -> Int {
        guard URL(string: url) != nil else {
            throw WebhookError.invalidURL(url)
        }
        sentWebhooks.append(SentWebhook(url: url, payload: payload))
        return 200
    }
}
