import Testing
import Foundation
@testable import K1s0TestHelper

@Suite("JwtTestHelper Tests")
struct JwtTestHelperTests {
    let helper = JwtTestHelper(secret: "test-secret")

    @Test func createAdminToken() throws {
        let token = try helper.createAdminToken()
        let parts = token.split(separator: ".")
        #expect(parts.count == 3)
        let claims = helper.decodeClaims(token: token)
        #expect(claims != nil)
        #expect(claims?.sub == "admin")
        #expect(claims?.roles == ["admin"])
    }

    @Test func createUserToken() throws {
        let token = try helper.createUserToken(userId: "user-123", roles: ["user"])
        let claims = helper.decodeClaims(token: token)
        #expect(claims != nil)
        #expect(claims?.sub == "user-123")
        #expect(claims?.roles == ["user"])
    }

    @Test func createTokenWithTenant() throws {
        let token = try helper.createToken(claims: TestClaims(
            sub: "svc", roles: ["service"], tenantId: "t-1"
        ))
        let claims = helper.decodeClaims(token: token)
        #expect(claims != nil)
        #expect(claims?.tenantId == "t-1")
    }

    @Test func decodeInvalidToken() {
        let claims = helper.decodeClaims(token: "invalid")
        #expect(claims == nil)
    }
}

@Suite("MockServerBuilder Tests")
struct MockServerBuilderTests {
    @Test func notificationServerWithHealth() {
        var builder = MockServerBuilder.notificationServer()
        builder = builder.withHealthOk()
        builder = builder.withSuccessResponse(path: "/send", body: "{\"id\":\"1\"}")
        let server = builder.build()

        let health = server.handle(method: "GET", path: "/health")
        #expect(health != nil)
        #expect(health?.status == 200)
        #expect(health?.body.contains("ok") == true)

        let send = server.handle(method: "POST", path: "/send")
        #expect(send != nil)
        #expect(send?.status == 200)

        #expect(server.requestCount == 2)
    }

    @Test func unknownRouteReturnsNil() {
        var builder = MockServerBuilder.ratelimitServer()
        builder = builder.withHealthOk()
        let server = builder.build()
        #expect(server.handle(method: "GET", path: "/nonexistent") == nil)
    }

    @Test func errorResponse() {
        var builder = MockServerBuilder.tenantServer()
        builder = builder.withErrorResponse(path: "/create", status: 500)
        let server = builder.build()
        let result = server.handle(method: "POST", path: "/create")
        #expect(result != nil)
        #expect(result?.status == 500)
        #expect(result?.body.contains("error") == true)
    }
}

@Suite("FixtureBuilder Tests")
struct FixtureBuilderTests {
    @Test func uuid() {
        let id = FixtureBuilder.uuid()
        #expect(id.count == 36)
        #expect(id.contains("-"))
    }

    @Test func email() {
        let email = FixtureBuilder.email()
        #expect(email.contains("@example.com"))
    }

    @Test func name() {
        let name = FixtureBuilder.name()
        #expect(name.hasPrefix("user-"))
    }

    @Test func intInRange() {
        for _ in 0..<100 {
            let val = FixtureBuilder.int(min: 10, max: 20)
            #expect(val >= 10)
            #expect(val < 20)
        }
    }

    @Test func intSameMinMax() {
        #expect(FixtureBuilder.int(min: 5, max: 5) == 5)
    }

    @Test func tenantId() {
        #expect(FixtureBuilder.tenantId().hasPrefix("tenant-"))
    }

    @Test func uniqueness() {
        let a = FixtureBuilder.uuid()
        let b = FixtureBuilder.uuid()
        #expect(a != b)
    }
}

@Suite("AssertionHelper Tests")
struct AssertionHelperTests {
    @Test func jsonContainsPartialMatch() throws {
        try AssertionHelper.assertJsonContains(
            "{\"id\":\"1\",\"status\":\"ok\",\"extra\":\"ignored\"}",
            "{\"id\":\"1\",\"status\":\"ok\"}")
    }

    @Test func jsonContainsNestedMatch() throws {
        try AssertionHelper.assertJsonContains(
            "{\"user\":{\"id\":\"1\",\"name\":\"test\"},\"status\":\"ok\"}",
            "{\"user\":{\"id\":\"1\"}}")
    }

    @Test func jsonContainsMismatchThrows() {
        #expect(throws: TestHelperError.self) {
            try AssertionHelper.assertJsonContains("{\"id\":\"1\"}", "{\"id\":\"2\"}")
        }
    }

    @Test func eventEmitted() throws {
        let events: [[String: Any]] = [
            ["type": "created", "id": "1"],
            ["type": "updated", "id": "2"],
        ]
        try AssertionHelper.assertEventEmitted(in: events, type: "created")
        try AssertionHelper.assertEventEmitted(in: events, type: "updated")
    }

    @Test func eventNotEmittedThrows() {
        let events: [[String: Any]] = [["type": "created"]]
        #expect(throws: TestHelperError.self) {
            try AssertionHelper.assertEventEmitted(in: events, type: "deleted")
        }
    }
}
