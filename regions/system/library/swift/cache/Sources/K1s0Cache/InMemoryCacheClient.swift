import Foundation

/// インメモリキャッシュクライアント。
public actor InMemoryCacheClient<T: Sendable>: CacheClient {
    public typealias Value = T

    private var storage: [String: CacheEntry<T>] = [:]

    public init() {}

    /// 指定されたキーの値を取得する。期限切れエントリは削除する。
    public func get(_ key: String) async throws -> T? {
        guard let entry = storage[key] else {
            return nil
        }
        if entry.isExpired {
            storage.removeValue(forKey: key)
            return nil
        }
        return entry.value
    }

    /// 指定されたキーに値を保存する。
    public func set(_ key: String, value: T, ttl: TimeInterval? = nil) async throws {
        let expiresAt = ttl.map { Date().addingTimeInterval($0) }
        storage[key] = CacheEntry(value: value, expiresAt: expiresAt)
    }

    /// 指定されたキーのエントリを削除する。
    public func delete(_ key: String) async throws {
        storage.removeValue(forKey: key)
    }

    /// 指定されたキーが存在し期限切れでないか確認する。
    public func exists(_ key: String) async -> Bool {
        guard let entry = storage[key] else { return false }
        if entry.isExpired {
            return false
        }
        return true
    }
}
