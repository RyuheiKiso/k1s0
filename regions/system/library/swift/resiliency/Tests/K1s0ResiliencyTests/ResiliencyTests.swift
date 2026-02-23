import Testing
import Foundation
@testable import K1s0Resiliency

@Suite("ResiliencyDecorator Tests")
struct ResiliencyDecoratorTests {

    @Test("Execute successfully")
    func executeSuccess() async throws {
        let decorator = ResiliencyDecorator(policy: ResiliencyPolicy())
        let result = try await decorator.execute { 42 }
        #expect(result == 42)
    }

    @Test("Retry on failure")
    func retryOnFailure() async throws {
        let policy = ResiliencyPolicy(
            retry: RetryConfig(
                maxAttempts: 3,
                baseDelay: .milliseconds(10),
                maxDelay: .milliseconds(100),
                jitter: false
            )
        )
        let decorator = ResiliencyDecorator(policy: policy)

        let counter = Counter()
        let result: Int = try await decorator.execute {
            let count = await counter.increment()
            if count < 3 {
                throw TestError.transient
            }
            return 99
        }

        #expect(result == 99)
        let finalCount = await counter.value
        #expect(finalCount == 3)
    }

    @Test("Max retries exceeded")
    func maxRetriesExceeded() async {
        let policy = ResiliencyPolicy(
            retry: RetryConfig(
                maxAttempts: 2,
                baseDelay: .milliseconds(1),
                maxDelay: .milliseconds(10),
                jitter: false
            )
        )
        let decorator = ResiliencyDecorator(policy: policy)

        do {
            let _: Int = try await decorator.execute {
                throw TestError.permanent
            }
            Issue.record("Expected error")
        } catch let error as ResiliencyError {
            switch error {
            case .maxRetriesExceeded(let attempts, _):
                #expect(attempts == 2)
            default:
                Issue.record("Expected maxRetriesExceeded, got \(error)")
            }
        } catch {
            Issue.record("Unexpected error: \(error)")
        }
    }

    @Test("Timeout")
    func timeout() async {
        let policy = ResiliencyPolicy(timeout: .milliseconds(50))
        let decorator = ResiliencyDecorator(policy: policy)

        do {
            let _: Int = try await decorator.execute {
                try await Task.sleep(for: .seconds(1))
                return 42
            }
            Issue.record("Expected timeout error")
        } catch let error as ResiliencyError {
            switch error {
            case .timeout:
                break
            default:
                Issue.record("Expected timeout, got \(error)")
            }
        } catch {
            // Task cancellation is acceptable here
        }
    }

    @Test("Circuit breaker opens after failures")
    func circuitBreakerOpens() async {
        let policy = ResiliencyPolicy(
            circuitBreaker: CircuitBreakerConfig(
                failureThreshold: 3,
                recoveryTimeout: .seconds(60),
                halfOpenMaxCalls: 1
            )
        )
        let decorator = ResiliencyDecorator(policy: policy)

        for _ in 0..<3 {
            do {
                let _: Int = try await decorator.execute {
                    throw TestError.transient
                }
            } catch {
                // expected
            }
        }

        do {
            let _: Int = try await decorator.execute { 42 }
            Issue.record("Expected circuit breaker open error")
        } catch let error as ResiliencyError {
            switch error {
            case .circuitBreakerOpen:
                break
            default:
                Issue.record("Expected circuitBreakerOpen, got \(error)")
            }
        } catch {
            Issue.record("Unexpected error: \(error)")
        }
    }
}

// MARK: - Helpers

private enum TestError: Error, Sendable {
    case transient
    case permanent
}

private actor Counter {
    var value: Int = 0

    func increment() -> Int {
        value += 1
        return value
    }
}
