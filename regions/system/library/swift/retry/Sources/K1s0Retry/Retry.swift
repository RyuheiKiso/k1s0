import Foundation

/// 指定した操作を設定に従ってリトライする。
///
/// - Parameters:
///   - operation: リトライする非同期操作。
///   - config: リトライ設定。
/// - Returns: 操作の結果。
/// - Throws: 最大試行回数に達した場合は `RetryError.maxAttemptsReached`。
public func withRetry<T: Sendable>(
    _ operation: @Sendable () async throws -> T,
    config: RetryConfig = .default
) async throws -> T {
    var delay = config.initialDelay

    for attempt in 1...config.maxAttempts {
        do {
            return try await operation()
        } catch {
            if attempt < config.maxAttempts {
                var sleepDuration = min(delay, config.maxDelay)
                if config.jitter {
                    sleepDuration *= Double.random(in: 0.5...1.5)
                    sleepDuration = min(sleepDuration, config.maxDelay)
                }
                try await Task.sleep(nanoseconds: UInt64(sleepDuration * 1_000_000_000))
                delay = min(delay * config.multiplier, config.maxDelay)
            }
        }
    }

    throw RetryError.maxAttemptsReached(config.maxAttempts)
}
