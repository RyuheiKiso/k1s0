import Foundation

public protocol SessionClient: Sendable {
    func create(req: CreateSessionRequest) async throws -> Session
    func get(id: String) async throws -> Session?
    func refresh(req: RefreshSessionRequest) async throws -> Session
    func revoke(id: String) async throws
    func listUserSessions(userId: String) async throws -> [Session]
    func revokeAll(userId: String) async throws -> Int
}

public actor InMemorySessionClient: SessionClient {
    private var sessions: [String: Session] = [:]

    public init() {}

    public func create(req: CreateSessionRequest) async throws -> Session {
        let now = Date()
        let session = Session(
            id: UUID().uuidString,
            userId: req.userId,
            token: UUID().uuidString,
            expiresAt: now.addingTimeInterval(Double(req.ttlSeconds)),
            createdAt: now,
            metadata: req.metadata
        )
        sessions[session.id] = session
        return session
    }

    public func get(id: String) async throws -> Session? {
        sessions[id]
    }

    public func refresh(req: RefreshSessionRequest) async throws -> Session {
        guard let existing = sessions[req.id] else {
            throw SessionError.sessionNotFound(id: req.id)
        }
        guard !existing.revoked else {
            throw SessionError.sessionRevoked(id: req.id)
        }
        let refreshed = Session(
            id: existing.id,
            userId: existing.userId,
            token: existing.token,
            expiresAt: Date().addingTimeInterval(Double(req.ttlSeconds)),
            createdAt: existing.createdAt,
            metadata: existing.metadata
        )
        sessions[req.id] = refreshed
        return refreshed
    }

    public func revoke(id: String) async throws {
        guard let existing = sessions[id] else {
            throw SessionError.sessionNotFound(id: id)
        }
        let revoked = Session(
            id: existing.id,
            userId: existing.userId,
            token: existing.token,
            expiresAt: existing.expiresAt,
            createdAt: existing.createdAt,
            revoked: true,
            metadata: existing.metadata
        )
        sessions[id] = revoked
    }

    public func listUserSessions(userId: String) async throws -> [Session] {
        sessions.values.filter { $0.userId == userId }
    }

    public func revokeAll(userId: String) async throws -> Int {
        let userSessions = sessions.values.filter { $0.userId == userId && !$0.revoked }
        for session in userSessions {
            let revoked = Session(
                id: session.id,
                userId: session.userId,
                token: session.token,
                expiresAt: session.expiresAt,
                createdAt: session.createdAt,
                revoked: true,
                metadata: session.metadata
            )
            sessions[session.id] = revoked
        }
        return userSessions.count
    }
}
