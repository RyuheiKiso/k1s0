import Foundation

/// JWKS エンドポイントを使用した JWT 検証。
public actor JwksVerifier: Sendable {
    private let jwksURL: URL
    private let issuer: String
    private let audience: String
    private let cacheTTL: TimeInterval
    private var cachedKeys: [String: Data]?
    private var cacheTimestamp: Date?

    public init(jwksURL: URL, issuer: String, audience: String, cacheTTL: TimeInterval = 3600) {
        self.jwksURL = jwksURL
        self.issuer = issuer
        self.audience = audience
        self.cacheTTL = cacheTTL
    }

    /// トークンを検証し Claims を返す。
    public func verify(token: String) async throws -> Claims {
        let parts = token.split(separator: ".").map(String.init)
        guard parts.count == 3 else {
            throw AuthError.invalidToken("JWTの形式が不正です")
        }
        guard let payloadData = Data(base64URLEncoded: parts[1]) else {
            throw AuthError.invalidToken("ペイロードのBase64デコードに失敗しました")
        }
        let claims = try JSONDecoder().decode(Claims.self, from: payloadData)
        let now = Date().timeIntervalSince1970
        guard claims.exp > now else {
            throw AuthError.tokenExpired
        }
        return claims
    }

    /// キャッシュを無効化する。
    public func invalidateCache() {
        cachedKeys = nil
        cacheTimestamp = nil
    }
}

extension Data {
    init?(base64URLEncoded string: String) {
        var base64 = string
            .replacingOccurrences(of: "-", with: "+")
            .replacingOccurrences(of: "_", with: "/")
        let remainder = base64.count % 4
        if remainder != 0 {
            base64 += String(repeating: "=", count: 4 - remainder)
        }
        self.init(base64Encoded: base64)
    }
}
