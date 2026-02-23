import Foundation
import Testing
@testable import K1s0RateLimitClient

@Suite("RateLimitClient Tests")
struct RateLimitClientTests {
    @Test("checkで許可が返ること")
    func testCheckAllowed() async throws {
        let client = InMemoryRateLimitClient()
        let status = try await client.check(key: "test-key", cost: 1)
        #expect(status.allowed == true)
        #expect(status.remaining == 99)
        #expect(status.retryAfterSecs == nil)
    }

    @Test("checkで制限超過が返ること")
    func testCheckDenied() async throws {
        let client = InMemoryRateLimitClient()
        await client.setPolicy(
            key: "limited",
            policy: RateLimitPolicy(key: "limited", limit: 2, windowSecs: 60, algorithm: "fixed_window")
        )

        _ = try await client.consume(key: "limited", cost: 2)
        let status = try await client.check(key: "limited", cost: 1)
        #expect(status.allowed == false)
        #expect(status.remaining == 0)
        #expect(status.retryAfterSecs == 60)
    }

    @Test("consumeで使用量を消費できること")
    func testConsume() async throws {
        let client = InMemoryRateLimitClient()
        let result = try await client.consume(key: "test-key", cost: 1)
        #expect(result.remaining == 99)
        let used = await client.getUsedCount(key: "test-key")
        #expect(used == 1)
    }

    @Test("consume制限超過でエラーが発生すること")
    func testConsumeExceedsLimit() async throws {
        let client = InMemoryRateLimitClient()
        await client.setPolicy(
            key: "small",
            policy: RateLimitPolicy(key: "small", limit: 1, windowSecs: 60, algorithm: "token_bucket")
        )

        _ = try await client.consume(key: "small", cost: 1)
        do {
            _ = try await client.consume(key: "small", cost: 1)
            #expect(Bool(false), "Expected error")
        } catch is RateLimitError {
            // expected
        }
    }

    @Test("getLimitでデフォルトポリシーが返ること")
    func testGetLimitDefault() async throws {
        let client = InMemoryRateLimitClient()
        let policy = try await client.getLimit(key: "unknown")
        #expect(policy.limit == 100)
        #expect(policy.windowSecs == 3600)
        #expect(policy.algorithm == "token_bucket")
    }

    @Test("getLimitでカスタムポリシーが返ること")
    func testGetLimitCustom() async throws {
        let client = InMemoryRateLimitClient()
        await client.setPolicy(
            key: "tenant:T1",
            policy: RateLimitPolicy(key: "tenant:T1", limit: 50, windowSecs: 1800, algorithm: "sliding_window")
        )

        let policy = try await client.getLimit(key: "tenant:T1")
        #expect(policy.key == "tenant:T1")
        #expect(policy.limit == 50)
        #expect(policy.algorithm == "sliding_window")
    }

    @Test("RateLimitStatusの全フィールドが正しいこと")
    func testStatusFields() {
        let status = RateLimitStatus(allowed: true, remaining: 50, resetAt: Date())
        #expect(status.allowed == true)
        #expect(status.remaining == 50)
        #expect(status.retryAfterSecs == nil)
    }

    @Test("RateLimitPolicyの全フィールドが正しいこと")
    func testPolicyFields() {
        let policy = RateLimitPolicy(key: "test", limit: 100, windowSecs: 3600, algorithm: "token_bucket")
        #expect(policy.key == "test")
        #expect(policy.limit == 100)
    }
}
