import Foundation

/// キャッシュクライアントプロトコル。
public protocol CacheClient<Value>: Sendable {
    associatedtype Value: Sendable

    /// 指定されたキーの値を取得する。
    func get(_ key: String) async throws -> Value?
    /// 指定されたキーに値を保存する。
    func set(_ key: String, value: Value, ttl: TimeInterval?) async throws
    /// 指定されたキーのエントリを削除する。
    func delete(_ key: String) async throws
    /// 指定されたキーが存在するか確認する。
    func exists(_ key: String) async -> Bool
}
