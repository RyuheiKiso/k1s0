import Foundation

public struct Event: Sendable {
    public let id: String
    public let eventType: String
    public let payload: [String: String]
    public let timestamp: Date

    public init(eventType: String, payload: [String: String]) {
        self.id = UUID().uuidString
        self.eventType = eventType
        self.payload = payload
        self.timestamp = Date()
    }
}

public typealias EventHandler = @Sendable (Event) async throws -> Void

public actor InMemoryEventBus {
    private var handlers: [String: [EventHandler]] = [:]

    public init() {}

    public func subscribe(_ eventType: String, handler: @escaping EventHandler) {
        handlers[eventType, default: []].append(handler)
    }

    public func unsubscribe(_ eventType: String) {
        handlers.removeValue(forKey: eventType)
    }

    public func publish(_ event: Event) async throws {
        for handler in handlers[event.eventType] ?? [] {
            try await handler(event)
        }
    }
}
