import Foundation

/// DLQ メッセージステータス。
public enum DlqStatus: String, Codable, Sendable {
    case pending = "pending"
    case retrying = "retrying"
    case resolved = "resolved"
    case dead = "dead"
}

/// DLQ メッセージ。
public struct DlqMessage: Codable, Sendable {
    public let id: String
    public let originalTopic: String
    public let errorMessage: String
    public let retryCount: Int
    public let maxRetries: Int
    public let status: DlqStatus
    public let createdAt: String
    public let lastRetryAt: String?

    enum CodingKeys: String, CodingKey {
        case id
        case originalTopic = "original_topic"
        case errorMessage = "error_message"
        case retryCount = "retry_count"
        case maxRetries = "max_retries"
        case status
        case createdAt = "created_at"
        case lastRetryAt = "last_retry_at"
    }
}

/// メッセージ一覧レスポンス。
public struct ListDlqMessagesResponse: Codable, Sendable {
    public let messages: [DlqMessage]
    public let total: Int
    public let page: Int
}

/// 再処理レスポンス。
public struct RetryDlqMessageResponse: Codable, Sendable {
    public let messageId: String
    public let status: DlqStatus

    enum CodingKeys: String, CodingKey {
        case messageId = "message_id"
        case status
    }
}
