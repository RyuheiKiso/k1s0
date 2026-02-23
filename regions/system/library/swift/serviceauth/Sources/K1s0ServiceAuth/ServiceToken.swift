import Foundation

/// OAuth2 アクセストークン。
public struct ServiceToken: Sendable {
    public let accessToken: String
    public let tokenType: String
    public let expiresIn: TimeInterval
    public let acquiredAt: Date

    public init(accessToken: String, tokenType: String = "Bearer", expiresIn: TimeInterval) {
        self.accessToken = accessToken
        self.tokenType = tokenType
        self.expiresIn = expiresIn
        self.acquiredAt = Date()
    }

    /// トークンが期限切れか判定する。
    public var isExpired: Bool {
        Date() >= acquiredAt.addingTimeInterval(expiresIn)
    }

    /// リフレッシュが必要か判定する。
    public func shouldRefresh(before seconds: TimeInterval = 120) -> Bool {
        Date() >= acquiredAt.addingTimeInterval(expiresIn - seconds)
    }

    /// Bearer ヘッダー値を返す。
    public var bearerHeader: String {
        "Bearer \(accessToken)"
    }
}
