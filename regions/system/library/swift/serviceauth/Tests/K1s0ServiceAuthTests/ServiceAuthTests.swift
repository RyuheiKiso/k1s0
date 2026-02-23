import Testing
@testable import K1s0ServiceAuth

@Suite("ServiceAuth Tests")
struct ServiceAuthTests {
    @Test("ServiceToken の期限切れ判定が正しいこと")
    func testServiceTokenExpiry() {
        let token = ServiceToken(accessToken: "test", expiresIn: 3600)
        #expect(!token.isExpired)
        #expect(!token.shouldRefresh(before: 120))
    }

    @Test("Bearer ヘッダーが正しいこと")
    func testBearerHeader() {
        let token = ServiceToken(accessToken: "my-token", expiresIn: 3600)
        #expect(token.bearerHeader == "Bearer my-token")
    }

    @Test("SpiffeId が正しく解析されること")
    func testSpiffeIdParsing() throws {
        let spiffe = try SpiffeId.parse("spiffe://k1s0.internal/ns/system/sa/auth-service")
        #expect(spiffe.trustDomain == "k1s0.internal")
        #expect(spiffe.namespace == "system")
        #expect(spiffe.serviceAccount == "auth-service")
    }

    @Test("SpiffeId のTierアクセス判定が正しいこと")
    func testSpiffeTierAccess() throws {
        let systemSpiffe = try SpiffeId.parse("spiffe://k1s0.internal/ns/system/sa/auth-service")
        #expect(systemSpiffe.allowsTierAccess(to: "service"))
        #expect(systemSpiffe.allowsTierAccess(to: "business"))

        let serviceSpiffe = try SpiffeId.parse("spiffe://k1s0.internal/ns/service/sa/order-service")
        #expect(!serviceSpiffe.allowsTierAccess(to: "system"))
        #expect(!serviceSpiffe.allowsTierAccess(to: "business"))
        #expect(serviceSpiffe.allowsTierAccess(to: "service"))
    }

    @Test("不正なSPIFFE URIはエラーになること")
    func testInvalidSpiffeUri() {
        #expect(throws: ServiceAuthError.self) {
            try SpiffeId.parse("https://invalid.example.com")
        }
    }

    @Test("ServiceAuthError の説明が含まれること")
    func testServiceAuthErrorDescription() {
        let error = ServiceAuthError.tokenAcquisition("network failed")
        #expect(error.description.contains("TOKEN_ACQUISITION_ERROR"))
    }
}
