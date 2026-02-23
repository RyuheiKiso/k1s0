import Testing
@testable import K1s0Auth

@Suite("Auth Tests")
struct AuthTests {
    @Test("ロールの判定が正しいこと")
    func testHasRole() {
        let claims = makeClaims(realmRoles: ["admin", "user"])
        #expect(RBAC.hasRole(claims, role: "admin"))
        #expect(!RBAC.hasRole(claims, role: "sys_admin"))
    }

    @Test("sys_admin は全権限を持つこと")
    func testSysAdminHasAllPermissions() {
        let claims = makeClaims(realmRoles: ["sys_admin"])
        #expect(RBAC.hasPermission(claims, resource: "orders", action: "read"))
        #expect(RBAC.hasPermission(claims, resource: "orders", action: "write"))
    }

    @Test("Tierアクセス判定が正しいこと")
    func testHasTierAccess() {
        let claims = makeClaims(tierAccess: ["system", "business"])
        #expect(RBAC.hasTierAccess(claims, tier: "system"))
        #expect(!RBAC.hasTierAccess(claims, tier: "service"))
    }

    @Test("AuthErrorの説明が含まれること")
    func testAuthErrorDescription() {
        let error = AuthError.invalidToken("署名が不正")
        #expect(error.description.contains("INVALID_TOKEN"))
        #expect(error.description.contains("署名が不正"))
    }

    // MARK: - ヘルパー

    private func makeClaims(
        realmRoles: [String] = [],
        tierAccess: [String] = []
    ) -> Claims {
        Claims(
            sub: "user-id",
            iss: "https://keycloak.example.com",
            exp: Date().timeIntervalSince1970 + 3600,
            iat: Date().timeIntervalSince1970,
            jti: nil,
            typ: "Bearer",
            azp: nil,
            scope: "openid",
            preferredUsername: "testuser",
            email: "test@example.com",
            realmAccess: RealmAccess(roles: realmRoles),
            resourceAccess: nil,
            tierAccess: tierAccess
        )
    }
}
