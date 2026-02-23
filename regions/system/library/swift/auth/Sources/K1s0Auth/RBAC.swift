/// ロールベースアクセス制御ヘルパー。
public enum RBAC: Sendable {
    /// グローバルロールを持つか判定する。
    public static func hasRole(_ claims: Claims, role: String) -> Bool {
        claims.realmRoles.contains(role)
    }

    /// リソースロールを持つか判定する。
    public static func hasResourceRole(_ claims: Claims, resource: String, role: String) -> Bool {
        claims.resourceRoles(for: resource).contains(role)
    }

    /// アクションの権限を持つか判定する。
    ///
    /// - sys_admin / admin → 全権限
    /// - リソース admin → 当該リソース全権限
    public static func hasPermission(_ claims: Claims, resource: String, action: String) -> Bool {
        if hasRole(claims, role: "sys_admin") || hasRole(claims, role: "admin") {
            return true
        }
        if hasResourceRole(claims, resource: resource, role: "admin") {
            return true
        }
        return hasResourceRole(claims, resource: resource, role: action)
    }

    /// Tierアクセスを持つか判定する。
    public static func hasTierAccess(_ claims: Claims, tier: String) -> Bool {
        claims.tierAccessList.contains(tier)
    }
}
