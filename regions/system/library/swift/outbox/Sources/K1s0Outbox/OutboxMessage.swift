import Foundation

/// アウトボックスメッセージのステータス。
public enum OutboxStatus: String, Sendable, Codable {
    case pending = "pending"
    case processing = "processing"
    case delivered = "delivered"
    case failed = "failed"
    case deadLetter = "dead_letter"
}

/// アウトボックスメッセージ。
public struct OutboxMessage: Sendable {
    public let id: UUID
    public let topic: String
    public let partitionKey: String
    public let payload: Data
    public private(set) var status: OutboxStatus
    public private(set) var retryCount: Int
    public let maxRetries: Int
    public private(set) var lastError: String?
    public let createdAt: Date
    public private(set) var processAfter: Date

    public init(topic: String, partitionKey: String, payload: Data, maxRetries: Int = 3) {
        self.id = UUID()
        self.topic = topic
        self.partitionKey = partitionKey
        self.payload = payload
        self.status = .pending
        self.retryCount = 0
        self.maxRetries = maxRetries
        self.lastError = nil
        self.createdAt = Date()
        self.processAfter = Date()
    }

    /// 処理中に遷移する。
    public mutating func markProcessing() {
        status = .processing
    }

    /// 配信完了に遷移する。
    public mutating func markDelivered() {
        status = .delivered
    }

    /// 失敗に遷移する（指数バックオフ）。
    public mutating func markFailed(error: String) {
        retryCount += 1
        lastError = error
        if retryCount >= maxRetries {
            status = .deadLetter
        } else {
            status = .failed
            // 指数バックオフ: 2^retryCount 秒後
            let backoffSeconds = pow(2.0, Double(retryCount))
            processAfter = Date().addingTimeInterval(backoffSeconds)
        }
    }

    /// 処理可能か判定する。
    public var isProcessable: Bool {
        (status == .pending || status == .failed) && Date() >= processAfter
    }
}
