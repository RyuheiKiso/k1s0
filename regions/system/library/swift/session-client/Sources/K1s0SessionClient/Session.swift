import Foundation

public struct Session: Sendable {
    public let id: String
    public let userId: String
    public let token: String
    public let expiresAt: Date
    public let createdAt: Date
    public let revoked: Bool
    public let metadata: [String: String]

    public init(
        id: String,
        userId: String,
        token: String,
        expiresAt: Date,
        createdAt: Date,
        revoked: Bool = false,
        metadata: [String: String] = [:]
    ) {
        self.id = id
        self.userId = userId
        self.token = token
        self.expiresAt = expiresAt
        self.createdAt = createdAt
        self.revoked = revoked
        self.metadata = metadata
    }
}

public struct CreateSessionRequest: Sendable {
    public let userId: String
    public let ttlSeconds: Int
    public let metadata: [String: String]

    public init(userId: String, ttlSeconds: Int, metadata: [String: String] = [:]) {
        self.userId = userId
        self.ttlSeconds = ttlSeconds
        self.metadata = metadata
    }
}

public struct RefreshSessionRequest: Sendable {
    public let id: String
    public let ttlSeconds: Int

    public init(id: String, ttlSeconds: Int) {
        self.id = id
        self.ttlSeconds = ttlSeconds
    }
}

public enum SessionError: Error, Sendable {
    case sessionNotFound(id: String)
    case sessionRevoked(id: String)
    case sessionExpired(id: String)
}
