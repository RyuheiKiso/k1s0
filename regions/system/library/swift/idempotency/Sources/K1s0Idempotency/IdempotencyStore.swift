import Foundation

/// 冪等性レコードのストアプロトコル。
public protocol IdempotencyStore: Sendable {
    /// 指定されたキーのレコードを取得する。
    func get(_ key: String) async throws -> IdempotencyRecord?
    /// レコードを保存する。
    func save(_ record: IdempotencyRecord) async throws
    /// 指定されたキーのレコードを削除する。
    func delete(_ key: String) async throws
}

/// インメモリ冪等性ストア。
public actor InMemoryIdempotencyStore: IdempotencyStore {
    private var storage: [String: IdempotencyRecord] = [:]

    public init() {}

    /// 指定されたキーのレコードを取得する。期限切れは nil を返す。
    public func get(_ key: String) async throws -> IdempotencyRecord? {
        guard let record = storage[key] else {
            return nil
        }
        if record.isExpired {
            storage.removeValue(forKey: key)
            return nil
        }
        return record
    }

    /// レコードを保存する。
    public func save(_ record: IdempotencyRecord) async throws {
        storage[record.key] = record
    }

    /// 指定されたキーのレコードを削除する。
    public func delete(_ key: String) async throws {
        storage.removeValue(forKey: key)
    }
}
