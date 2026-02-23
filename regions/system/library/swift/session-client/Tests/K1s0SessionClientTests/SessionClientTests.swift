import Testing
import Foundation
@testable import K1s0SessionClient

@Suite("SessionClient Tests")
struct SessionClientTests {
    @Test("セッションを作成できること")
    func testCreate() async throws {
        let client = InMemorySessionClient()
        let req = CreateSessionRequest(userId: "user-1", ttlSeconds: 3600)
        let session = try await client.create(req: req)
        #expect(session.userId == "user-1")
        #expect(!session.id.isEmpty)
        #expect(!session.token.isEmpty)
        #expect(session.revoked == false)
    }

    @Test("セッションを取得できること")
    func testGet() async throws {
        let client = InMemorySessionClient()
        let req = CreateSessionRequest(userId: "user-1", ttlSeconds: 3600)
        let created = try await client.create(req: req)

        let retrieved = try await client.get(id: created.id)
        #expect(retrieved?.id == created.id)
        #expect(retrieved?.userId == "user-1")
    }

    @Test("存在しないセッションはnilを返すこと")
    func testGetNotFound() async throws {
        let client = InMemorySessionClient()
        let result = try await client.get(id: "nonexistent")
        #expect(result == nil)
    }

    @Test("セッションをリフレッシュできること")
    func testRefresh() async throws {
        let client = InMemorySessionClient()
        let req = CreateSessionRequest(userId: "user-1", ttlSeconds: 60)
        let created = try await client.create(req: req)

        let refreshReq = RefreshSessionRequest(id: created.id, ttlSeconds: 7200)
        let refreshed = try await client.refresh(req: refreshReq)
        #expect(refreshed.id == created.id)
        #expect(refreshed.expiresAt > created.expiresAt)
    }

    @Test("存在しないセッションのリフレッシュでエラーになること")
    func testRefreshNotFound() async throws {
        let client = InMemorySessionClient()
        let req = RefreshSessionRequest(id: "nonexistent", ttlSeconds: 3600)
        do {
            _ = try await client.refresh(req: req)
            #expect(Bool(false), "Should have thrown")
        } catch is SessionError {
            // expected
        }
    }

    @Test("セッションを取り消しできること")
    func testRevoke() async throws {
        let client = InMemorySessionClient()
        let req = CreateSessionRequest(userId: "user-1", ttlSeconds: 3600)
        let created = try await client.create(req: req)

        try await client.revoke(id: created.id)
        let revoked = try await client.get(id: created.id)
        #expect(revoked?.revoked == true)
    }

    @Test("取り消し済みセッションのリフレッシュでエラーになること")
    func testRefreshRevoked() async throws {
        let client = InMemorySessionClient()
        let req = CreateSessionRequest(userId: "user-1", ttlSeconds: 3600)
        let created = try await client.create(req: req)
        try await client.revoke(id: created.id)

        let refreshReq = RefreshSessionRequest(id: created.id, ttlSeconds: 3600)
        do {
            _ = try await client.refresh(req: refreshReq)
            #expect(Bool(false), "Should have thrown")
        } catch is SessionError {
            // expected
        }
    }

    @Test("ユーザーのセッション一覧を取得できること")
    func testListUserSessions() async throws {
        let client = InMemorySessionClient()
        _ = try await client.create(req: CreateSessionRequest(userId: "user-1", ttlSeconds: 3600))
        _ = try await client.create(req: CreateSessionRequest(userId: "user-1", ttlSeconds: 3600))
        _ = try await client.create(req: CreateSessionRequest(userId: "user-2", ttlSeconds: 3600))

        let sessions = try await client.listUserSessions(userId: "user-1")
        #expect(sessions.count == 2)
    }

    @Test("ユーザーの全セッションを取り消しできること")
    func testRevokeAll() async throws {
        let client = InMemorySessionClient()
        _ = try await client.create(req: CreateSessionRequest(userId: "user-1", ttlSeconds: 3600))
        _ = try await client.create(req: CreateSessionRequest(userId: "user-1", ttlSeconds: 3600))

        let count = try await client.revokeAll(userId: "user-1")
        #expect(count == 2)

        let sessions = try await client.listUserSessions(userId: "user-1")
        #expect(sessions.allSatisfy { $0.revoked })
    }

    @Test("メタデータ付きセッションを作成できること")
    func testCreateWithMetadata() async throws {
        let client = InMemorySessionClient()
        let req = CreateSessionRequest(userId: "user-1", ttlSeconds: 3600, metadata: ["device": "mobile"])
        let session = try await client.create(req: req)
        #expect(session.metadata["device"] == "mobile")
    }

    @Test("SessionErrorの各バリアント")
    func testSessionErrors() {
        let err1 = SessionError.sessionNotFound(id: "s-1")
        if case .sessionNotFound(let id) = err1 {
            #expect(id == "s-1")
        }

        let err2 = SessionError.sessionRevoked(id: "s-2")
        if case .sessionRevoked(let id) = err2 {
            #expect(id == "s-2")
        }

        let err3 = SessionError.sessionExpired(id: "s-3")
        if case .sessionExpired(let id) = err3 {
            #expect(id == "s-3")
        }
    }
}
