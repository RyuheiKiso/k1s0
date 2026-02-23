import Foundation

/// JWT Claims。
public struct Claims: Codable, Sendable {
    public let sub: String
    public let iss: String
    public let exp: TimeInterval
    public let iat: TimeInterval
    public let jti: String?
    public let typ: String?
    public let azp: String?
    public let scope: String?
    public let preferredUsername: String?
    public let email: String?
    public let realmAccess: RealmAccess?
    public let resourceAccess: [String: ResourceAccess]?
    public let tierAccess: [String]?

    enum CodingKeys: String, CodingKey {
        case sub, iss, exp, iat, jti, typ, azp, scope, email
        case preferredUsername = "preferred_username"
        case realmAccess = "realm_access"
        case resourceAccess = "resource_access"
        case tierAccess = "tier_access"
    }

    /// レルムロール一覧を返す。
    public var realmRoles: [String] {
        realmAccess?.roles ?? []
    }

    /// リソースごとのロール一覧を返す。
    public func resourceRoles(for resource: String) -> [String] {
        resourceAccess?[resource]?.roles ?? []
    }

    /// Tierアクセス一覧を返す。
    public var tierAccessList: [String] {
        tierAccess ?? []
    }
}

/// レルムアクセス情報。
public struct RealmAccess: Codable, Sendable {
    public let roles: [String]
}

/// リソースアクセス情報。
public struct ResourceAccess: Codable, Sendable {
    public let roles: [String]
}
