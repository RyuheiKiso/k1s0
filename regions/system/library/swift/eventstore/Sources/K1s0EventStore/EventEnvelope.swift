import Foundation

/// イベントをラップするエンベロープ。メタデータとペイロードを持つ。
public struct EventEnvelope: Sendable {
    /// イベントの一意識別子。
    public let eventId: String
    /// イベントが属するストリームID。
    public let streamId: StreamId
    /// イベントの種類。
    public let eventType: String
    /// イベントのペイロード（任意の構造化データ）。
    public let payload: [String: any Sendable]
    /// ストリーム内でのバージョン（連番）。
    public let version: Int
    /// イベントが発生した日時。
    public let occurredAt: Date

    public init(
        eventId: String,
        streamId: StreamId,
        eventType: String,
        payload: [String: any Sendable],
        version: Int,
        occurredAt: Date = Date()
    ) {
        self.eventId = eventId
        self.streamId = streamId
        self.eventType = eventType
        self.payload = payload
        self.version = version
        self.occurredAt = occurredAt
    }
}
