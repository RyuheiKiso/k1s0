import Foundation

/// テスト用 JWT クレーム。
public struct TestClaims: Sendable, Codable {
    public let sub: String
    public let roles: [String]
    public let tenantId: String?
    public let iat: Int
    public let exp: Int

    enum CodingKeys: String, CodingKey {
        case sub, roles, iat, exp
        case tenantId = "tenant_id"
    }

    public init(
        sub: String,
        roles: [String] = [],
        tenantId: String? = nil,
        expiresIn: Int = 3600
    ) {
        let now = Int(Date().timeIntervalSince1970)
        self.sub = sub
        self.roles = roles
        self.tenantId = tenantId
        self.iat = now
        self.exp = now + expiresIn
    }
}
