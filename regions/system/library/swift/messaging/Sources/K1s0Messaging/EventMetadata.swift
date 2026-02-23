import Foundation

/// メッセージメタデータ。
public struct EventMetadata: Codable, Sendable {
    public let eventId: String
    public let eventType: String
    public let source: String
    public let timestamp: Date
    public let traceId: String?
    public let correlationId: String?
    public let schemaVersion: Int

    public init(eventType: String, source: String) {
        self.eventId = UUID().uuidString.lowercased()
        self.eventType = eventType
        self.source = source
        self.timestamp = Date()
        self.traceId = nil
        self.correlationId = nil
        self.schemaVersion = 1
    }

    private init(
        eventId: String, eventType: String, source: String,
        timestamp: Date, traceId: String?, correlationId: String?, schemaVersion: Int
    ) {
        self.eventId = eventId
        self.eventType = eventType
        self.source = source
        self.timestamp = timestamp
        self.traceId = traceId
        self.correlationId = correlationId
        self.schemaVersion = schemaVersion
    }

    public func withTraceId(_ traceId: String) -> EventMetadata {
        EventMetadata(
            eventId: eventId, eventType: eventType, source: source,
            timestamp: timestamp, traceId: traceId, correlationId: correlationId, schemaVersion: schemaVersion
        )
    }

    public func withCorrelationId(_ correlationId: String) -> EventMetadata {
        EventMetadata(
            eventId: eventId, eventType: eventType, source: source,
            timestamp: timestamp, traceId: traceId, correlationId: correlationId, schemaVersion: schemaVersion
        )
    }

    enum CodingKeys: String, CodingKey {
        case eventId = "event_id"
        case eventType = "event_type"
        case source, timestamp
        case traceId = "trace_id"
        case correlationId = "correlation_id"
        case schemaVersion = "schema_version"
    }
}
