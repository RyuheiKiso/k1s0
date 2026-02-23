import Foundation

/// 冪等性レコード。リクエストの処理状態を追跡する。
public struct IdempotencyRecord: Sendable {
    /// 冪等性キー。
    public let key: String
    /// 現在の処理状態。
    public var status: IdempotencyStatus
    /// 完了時のレスポンスデータ。
    public var response: Data?
    /// レコードの作成日時。
    public let createdAt: Date
    /// レコードの有効期限。nil の場合は期限なし。
    public let expiresAt: Date?

    public init(
        key: String,
        status: IdempotencyStatus = .pending,
        response: Data? = nil,
        createdAt: Date = Date(),
        expiresAt: Date? = nil
    ) {
        self.key = key
        self.status = status
        self.response = response
        self.createdAt = createdAt
        self.expiresAt = expiresAt
    }

    /// レコードが期限切れかどうかを返す。
    public var isExpired: Bool {
        guard let expiresAt else { return false }
        return Date() >= expiresAt
    }
}
