import Foundation
import Testing
@testable import K1s0QuotaClient

@Suite("QuotaClient Tests")
struct QuotaClientTests {

    @Test("check returns allowed for within-limit request")
    func checkAllowed() async throws {
        let client = InMemoryQuotaClient()
        let status = try await client.check(quotaId: "q1", amount: 100)
        #expect(status.allowed == true)
        #expect(status.remaining == 1000)
        #expect(status.limit == 1000)
    }

    @Test("check returns not allowed when exceeded")
    func checkExceeded() async throws {
        let client = InMemoryQuotaClient()
        _ = try await client.increment(quotaId: "q1", amount: 900)
        let status = try await client.check(quotaId: "q1", amount: 200)
        #expect(status.allowed == false)
        #expect(status.remaining == 100)
    }

    @Test("increment accumulates usage")
    func incrementAccumulates() async throws {
        let client = InMemoryQuotaClient()
        _ = try await client.increment(quotaId: "q1", amount: 300)
        let usage = try await client.increment(quotaId: "q1", amount: 200)
        #expect(usage.used == 500)
        #expect(usage.limit == 1000)
    }

    @Test("getUsage returns current usage")
    func getUsage() async throws {
        let client = InMemoryQuotaClient()
        _ = try await client.increment(quotaId: "q1", amount: 100)
        let usage = try await client.getUsage(quotaId: "q1")
        #expect(usage.used == 100)
        #expect(usage.quotaId == "q1")
    }

    @Test("getPolicy returns default policy")
    func getPolicyDefault() async throws {
        let client = InMemoryQuotaClient()
        let policy = try await client.getPolicy(quotaId: "q1")
        #expect(policy.quotaId == "q1")
        #expect(policy.limit == 1000)
        #expect(policy.period == .daily)
        #expect(policy.resetStrategy == "fixed")
    }

    @Test("getPolicy returns custom policy")
    func getPolicyCustom() async throws {
        let client = InMemoryQuotaClient()
        await client.setPolicy("q1", policy: QuotaPolicy(
            quotaId: "q1", limit: 5000, period: .monthly, resetStrategy: "sliding"
        ))
        let policy = try await client.getPolicy(quotaId: "q1")
        #expect(policy.limit == 5000)
        #expect(policy.period == .monthly)
    }

    @Test("QuotaStatus stores all fields")
    func quotaStatusFields() {
        let now = Date()
        let status = QuotaStatus(allowed: true, remaining: 500, limit: 1000, resetAt: now)
        #expect(status.allowed == true)
        #expect(status.remaining == 500)
    }

    @Test("QuotaUsage stores all fields")
    func quotaUsageFields() {
        let now = Date()
        let usage = QuotaUsage(quotaId: "q1", used: 100, limit: 1000, period: .daily, resetAt: now)
        #expect(usage.quotaId == "q1")
        #expect(usage.used == 100)
    }
}
