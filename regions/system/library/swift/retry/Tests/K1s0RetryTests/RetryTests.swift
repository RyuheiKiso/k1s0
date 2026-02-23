import Testing
@testable import K1s0Retry

/// スレッドセーフなカウンター（Swift 6 の Sendable クロージャ制約を満たすため）。
private actor Counter {
    var count: Int = 0
    func increment() { count += 1 }
    func get() -> Int { count }
}

@Suite("Retry Tests")
struct RetryTests {
    @Test("withRetry が最初の試行で成功すること")
    func testSucceedsOnFirstAttempt() async throws {
        let result = try await withRetry({
            return 42
        }, config: RetryConfig(maxAttempts: 3))
        #expect(result == 42)
    }

    @Test("withRetry が失敗後にリトライして成功すること")
    func testRetriesOnFailureAndSucceeds() async throws {
        let counter = Counter()
        let result = try await withRetry({
            await counter.increment()
            let count = await counter.get()
            if count < 3 {
                throw RetryError.cancelled
            }
            return "success"
        }, config: RetryConfig(maxAttempts: 5, initialDelay: 0.0))
        #expect(result == "success")
        let finalCount = await counter.get()
        #expect(finalCount == 3)
    }

    @Test("withRetry が最大試行回数に達したら RetryError をスローすること")
    func testThrowsAfterMaxAttempts() async throws {
        let counter = Counter()
        do {
            _ = try await withRetry({
                await counter.increment()
                throw RetryError.cancelled
            }, config: RetryConfig(maxAttempts: 3, initialDelay: 0.0))
            Issue.record("エラーがスローされるべき")
        } catch let error as RetryError {
            switch error {
            case .maxAttemptsReached(let attempts):
                #expect(attempts == 3)
            default:
                Issue.record("maxAttemptsReached エラーが期待される")
            }
        }
        let finalCount = await counter.get()
        #expect(finalCount == 3)
    }

    @Test("CircuitBreaker が初期状態でクローズであること")
    func testCircuitBreakerStartsClosed() async {
        let breaker = CircuitBreaker()
        let state = await breaker.state
        #expect(state == .closed)
        let isOpen = await breaker.isOpen()
        #expect(!isOpen)
    }

    @Test("CircuitBreaker が閾値の失敗後にオープンになること")
    func testCircuitBreakerOpensAfterThresholdFailures() async {
        let config = CircuitBreakerConfig(failureThreshold: 3, successThreshold: 2, timeout: 60.0)
        let breaker = CircuitBreaker(config: config)

        await breaker.recordFailure()
        await breaker.recordFailure()
        let stateBefore = await breaker.state
        #expect(stateBefore == .closed)

        await breaker.recordFailure()
        let stateAfter = await breaker.state
        #expect(stateAfter == .open)
        let isOpen = await breaker.isOpen()
        #expect(isOpen)
    }
}
