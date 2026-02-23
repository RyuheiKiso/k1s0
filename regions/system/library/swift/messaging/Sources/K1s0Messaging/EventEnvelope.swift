import Foundation

/// Kafka に発行するメッセージのエンベロープ。
public struct EventEnvelope: Sendable {
    public let topic: String
    public let key: String
    public let payload: Data
    public let headers: [(String, Data)]

    public init(topic: String, key: String, payload: Data, headers: [(String, Data)] = []) {
        self.topic = topic
        self.key = key
        self.payload = payload
        self.headers = headers
    }

    /// JSON ペイロードのエンベロープを生成する。
    public static func json<T: Encodable>(
        topic: String, key: String, payload: T
    ) throws -> EventEnvelope {
        let data = try JSONEncoder().encode(payload)
        return EventEnvelope(topic: topic, key: key, payload: data)
    }
}
