import Foundation

/// サービス認証設定。
public struct ServiceAuthConfig: Sendable {
    public let tokenEndpoint: String
    public let clientId: String
    public let clientSecret: String
    public let jwksURI: String?
    public let refreshBeforeSeconds: TimeInterval
    public let timeoutSeconds: TimeInterval

    public init(
        tokenEndpoint: String,
        clientId: String,
        clientSecret: String,
        jwksURI: String? = nil,
        refreshBeforeSeconds: TimeInterval = 120,
        timeoutSeconds: TimeInterval = 10
    ) {
        self.tokenEndpoint = tokenEndpoint
        self.clientId = clientId
        self.clientSecret = clientSecret
        self.jwksURI = jwksURI
        self.refreshBeforeSeconds = refreshBeforeSeconds
        self.timeoutSeconds = timeoutSeconds
    }
}
