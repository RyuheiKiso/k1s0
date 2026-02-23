import Foundation

public struct LockGuard: Sendable {
    public let key: String
    public let token: String

    public init(key: String, token: String) {
        self.key = key
        self.token = token
    }
}

public enum LockError: Error, Sendable {
    case alreadyLocked(String)
    case tokenMismatch
    case notFound(String)
}

public protocol DistributedLock: Sendable {
    func acquire(_ key: String, ttl: TimeInterval) async throws -> LockGuard
    func release(_ guard: LockGuard) async throws
    func isLocked(_ key: String) async -> Bool
}

public actor InMemoryDistributedLock: DistributedLock {
    private struct Entry {
        let token: String
        let expiresAt: Date
    }

    private var locks: [String: Entry] = [:]

    public init() {}

    public func acquire(_ key: String, ttl: TimeInterval) async throws -> LockGuard {
        cleanExpired()
        if locks[key] != nil {
            throw LockError.alreadyLocked(key)
        }
        let token = UUID().uuidString
        locks[key] = Entry(token: token, expiresAt: Date().addingTimeInterval(ttl))
        return LockGuard(key: key, token: token)
    }

    public func release(_ lockGuard: LockGuard) async throws {
        guard let entry = locks[lockGuard.key] else {
            throw LockError.notFound(lockGuard.key)
        }
        guard entry.token == lockGuard.token else {
            throw LockError.tokenMismatch
        }
        locks.removeValue(forKey: lockGuard.key)
    }

    public func isLocked(_ key: String) async -> Bool {
        cleanExpired()
        return locks[key] != nil
    }

    private func cleanExpired() {
        let now = Date()
        locks = locks.filter { $0.value.expiresAt > now }
    }
}
