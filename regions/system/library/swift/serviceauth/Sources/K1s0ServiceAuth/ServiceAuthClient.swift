import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

/// サービス間認証クライアントプロトコル。
public protocol ServiceAuthClient: Sendable {
    func getToken() async throws -> ServiceToken
    func getCachedToken() async throws -> String
}

/// URLSession を使用したサービス認証クライアント。
public actor URLSessionServiceAuthClient: ServiceAuthClient {
    private let config: ServiceAuthConfig
    private var cachedToken: ServiceToken?

    public init(config: ServiceAuthConfig) {
        self.config = config
    }

    /// Client Credentials フローでトークンを取得する。
    public func getToken() async throws -> ServiceToken {
        guard let url = URL(string: config.tokenEndpoint) else {
            throw ServiceAuthError.tokenAcquisition("不正なエンドポイントURL: \(config.tokenEndpoint)")
        }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = config.timeoutSeconds
        let body = "grant_type=client_credentials&client_id=\(config.clientId)&client_secret=\(config.clientSecret)"
        request.httpBody = body.data(using: .utf8)

        let (data, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else {
            throw ServiceAuthError.tokenAcquisition("トークン取得に失敗しました")
        }
        let tokenResponse = try JSONDecoder().decode(TokenResponse.self, from: data)
        let token = ServiceToken(
            accessToken: tokenResponse.accessToken,
            tokenType: tokenResponse.tokenType,
            expiresIn: TimeInterval(tokenResponse.expiresIn)
        )
        cachedToken = token
        return token
    }

    /// キャッシュされたトークンを返す（期限切れ前にリフレッシュ）。
    public func getCachedToken() async throws -> String {
        if let token = cachedToken, !token.shouldRefresh(before: config.refreshBeforeSeconds) {
            return token.accessToken
        }
        let token = try await getToken()
        return token.accessToken
    }
}

private struct TokenResponse: Decodable {
    let accessToken: String
    let tokenType: String
    let expiresIn: Int

    enum CodingKeys: String, CodingKey {
        case accessToken = "access_token"
        case tokenType = "token_type"
        case expiresIn = "expires_in"
    }
}
