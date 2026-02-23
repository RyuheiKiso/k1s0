import Testing
@testable import K1s0CircuitBreaker

enum TestError: Error {
    case simulated
}

@Suite("CircuitBreaker Tests")
struct CircuitBreakerTests {
    @Test("初期状態がclosedであること")
    func testInitialState() async {
        let cb = CircuitBreaker(config: CircuitBreakerConfig(failureThreshold: 3, successThreshold: 2, timeout: 1.0))
        let state = await cb.currentState()
        #expect(state == .closed)
    }

    @Test("閾値に達するとopenになること")
    func testOpenOnThreshold() async {
        let cb = CircuitBreaker(config: CircuitBreakerConfig(failureThreshold: 2, successThreshold: 1, timeout: 1.0))
        await cb.recordFailure()
        await cb.recordFailure()
        let state = await cb.currentState()
        #expect(state == .open)
    }

    @Test("open状態でcallがエラーになること")
    func testCallWhenOpen() async {
        let cb = CircuitBreaker(config: CircuitBreakerConfig(failureThreshold: 1, successThreshold: 1, timeout: 60.0))
        await cb.recordFailure()
        let isOpen = await cb.isOpen()
        #expect(isOpen)

        do {
            let _: Int = try await cb.call { 42 }
            #expect(Bool(false), "Should have thrown")
        } catch is CircuitBreakerError {
            // expected
        } catch {
            #expect(Bool(false), "Unexpected error: \(error)")
        }
    }

    @Test("closed状態でcallが成功すること")
    func testCallWhenClosed() async throws {
        let cb = CircuitBreaker(config: CircuitBreakerConfig(failureThreshold: 3, successThreshold: 2, timeout: 1.0))
        let result = try await cb.call { 42 }
        #expect(result == 42)
    }

    @Test("成功記録で失敗カウントがリセットされること")
    func testSuccessResets() async {
        let cb = CircuitBreaker(config: CircuitBreakerConfig(failureThreshold: 3, successThreshold: 1, timeout: 1.0))
        await cb.recordFailure()
        await cb.recordFailure()
        await cb.recordSuccess()
        await cb.recordFailure()
        let state = await cb.currentState()
        #expect(state == .closed)
    }

    @Test("タイムアウト後にhalfOpenへ遷移すること")
    func testTimeoutTransition() async throws {
        let cb = CircuitBreaker(config: CircuitBreakerConfig(failureThreshold: 1, successThreshold: 1, timeout: 0.01))
        await cb.recordFailure()
        try await Task.sleep(nanoseconds: 20_000_000) // 20ms
        let result = try await cb.call { "ok" }
        #expect(result == "ok")
        let state = await cb.currentState()
        #expect(state == .closed)
    }
}
