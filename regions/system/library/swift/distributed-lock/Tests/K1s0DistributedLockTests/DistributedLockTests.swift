import Testing
@testable import K1s0DistributedLock

@Suite("DistributedLock Tests")
struct DistributedLockTests {
    @Test("ロックの取得と解放が正しく動作すること")
    func testAcquireAndRelease() async throws {
        let lock = InMemoryDistributedLock()
        let guard1 = try await lock.acquire("resource-1", ttl: 10.0)
        #expect(guard1.key == "resource-1")

        let locked = await lock.isLocked("resource-1")
        #expect(locked)

        try await lock.release(guard1)
        let afterRelease = await lock.isLocked("resource-1")
        #expect(!afterRelease)
    }

    @Test("二重ロックがエラーになること")
    func testDoubleLock() async throws {
        let lock = InMemoryDistributedLock()
        _ = try await lock.acquire("resource-1", ttl: 10.0)

        do {
            _ = try await lock.acquire("resource-1", ttl: 10.0)
            #expect(Bool(false), "Should have thrown")
        } catch is LockError {
            // expected
        }
    }

    @Test("異なるトークンでの解放がエラーになること")
    func testTokenMismatch() async throws {
        let lock = InMemoryDistributedLock()
        _ = try await lock.acquire("resource-1", ttl: 10.0)
        let fakeGuard = LockGuard(key: "resource-1", token: "wrong-token")

        do {
            try await lock.release(fakeGuard)
            #expect(Bool(false), "Should have thrown")
        } catch is LockError {
            // expected
        }
    }

    @Test("存在しないロックの解放がエラーになること")
    func testReleaseNotFound() async throws {
        let lock = InMemoryDistributedLock()
        let fakeGuard = LockGuard(key: "nonexistent", token: "token")

        do {
            try await lock.release(fakeGuard)
            #expect(Bool(false), "Should have thrown")
        } catch is LockError {
            // expected
        }
    }

    @Test("TTL期限切れでロックが自動解放されること")
    func testTTLExpiry() async throws {
        let lock = InMemoryDistributedLock()
        _ = try await lock.acquire("resource-1", ttl: -1.0) // already expired
        let locked = await lock.isLocked("resource-1")
        #expect(!locked)
    }
}
