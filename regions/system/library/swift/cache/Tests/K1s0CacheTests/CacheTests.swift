import Testing
@testable import K1s0Cache
import Foundation

@Suite("Cache Tests")
struct CacheTests {
    @Test("値をセットして取得できること")
    func testSetAndGet() async throws {
        let cache = InMemoryCacheClient<String>()
        try await cache.set("key1", value: "hello", ttl: nil)
        let result = try await cache.get("key1")
        #expect(result == "hello")
    }

    @Test("存在しないキーを取得すると nil が返ること")
    func testGetNonexistentReturnsNil() async throws {
        let cache = InMemoryCacheClient<String>()
        let result = try await cache.get("missing")
        #expect(result == nil)
    }

    @Test("エントリを削除できること")
    func testDeleteEntry() async throws {
        let cache = InMemoryCacheClient<Int>()
        try await cache.set("key1", value: 42, ttl: nil)
        let before = try await cache.get("key1")
        #expect(before == 42)

        try await cache.delete("key1")
        let after = try await cache.get("key1")
        #expect(after == nil)
    }

    @Test("TTL が切れたエントリは取得できないこと")
    func testTTLExpiry() async throws {
        let cache = InMemoryCacheClient<String>()
        try await cache.set("key1", value: "expired", ttl: -1.0)
        let result = try await cache.get("key1")
        #expect(result == nil)
    }

    @Test("exists が正しく動作すること")
    func testExists() async throws {
        let cache = InMemoryCacheClient<String>()
        let before = await cache.exists("key1")
        #expect(!before)

        try await cache.set("key1", value: "value", ttl: nil)
        let after = await cache.exists("key1")
        #expect(after)

        try await cache.delete("key1")
        let deleted = await cache.exists("key1")
        #expect(!deleted)
    }
}
