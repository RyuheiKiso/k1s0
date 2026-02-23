import Foundation

/// イベントプロデューサープロトコル。
public protocol EventProducer: Sendable {
    func publish(_ envelope: EventEnvelope) async throws
    func publishBatch(_ envelopes: [EventEnvelope]) async throws
}

/// イベントコンシューマープロトコル。
public protocol EventConsumer: Sendable {
    func receive() async throws -> ConsumedMessage
    func commit(_ message: ConsumedMessage) async throws
}

/// 受信メッセージ。
public struct ConsumedMessage: Sendable {
    public let topic: String
    public let partition: Int32
    public let offset: Int64
    public let key: Data?
    public let payload: Data

    public init(topic: String, partition: Int32, offset: Int64, key: Data?, payload: Data) {
        self.topic = topic
        self.partition = partition
        self.offset = offset
        self.key = key
        self.payload = payload
    }

    /// JSON デシリアライズする。
    public func deserializeJSON<T: Decodable>(as type: T.Type) throws -> T {
        try JSONDecoder().decode(type, from: payload)
    }
}

/// テスト用 NoOp プロデューサー。
public struct NoOpEventProducer: EventProducer {
    public init() {}
    public func publish(_ envelope: EventEnvelope) async throws {}
    public func publishBatch(_ envelopes: [EventEnvelope]) async throws {}
}
