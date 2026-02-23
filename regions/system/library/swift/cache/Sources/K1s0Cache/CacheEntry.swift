import Foundation

/// キャッシュエントリ。値と有効期限を持つ。
public struct CacheEntry<T: Sendable>: Sendable {
    /// キャッシュされた値。
    public let value: T
    /// 有効期限。nil の場合は期限なし。
    public let expiresAt: Date?

    public init(value: T, expiresAt: Date? = nil) {
        self.value = value
        self.expiresAt = expiresAt
    }

    /// エントリが期限切れかどうかを返す。
    public var isExpired: Bool {
        guard let expiresAt else { return false }
        return Date() >= expiresAt
    }
}
