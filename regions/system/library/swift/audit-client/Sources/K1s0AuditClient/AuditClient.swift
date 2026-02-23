import Foundation

public struct AuditEvent: Sendable {
    public let id: String
    public let tenantId: String
    public let actorId: String
    public let action: String
    public let resourceType: String
    public let resourceId: String
    public let timestamp: Date

    public init(tenantId: String, actorId: String, action: String, resourceType: String, resourceId: String) {
        self.id = UUID().uuidString
        self.tenantId = tenantId
        self.actorId = actorId
        self.action = action
        self.resourceType = resourceType
        self.resourceId = resourceId
        self.timestamp = Date()
    }
}

public protocol AuditClient: Sendable {
    func record(_ event: AuditEvent) async throws
    func flush() async throws -> [AuditEvent]
}

public actor BufferedAuditClient: AuditClient {
    private var buffer: [AuditEvent] = []

    public init() {}

    public func record(_ event: AuditEvent) async throws {
        buffer.append(event)
    }

    public func flush() async throws -> [AuditEvent] {
        defer { buffer.removeAll() }
        return buffer
    }
}
