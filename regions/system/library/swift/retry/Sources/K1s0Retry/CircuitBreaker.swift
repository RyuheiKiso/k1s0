import Foundation

/// サーキットブレーカーの状態。
public enum CircuitBreakerState: Sendable, Equatable {
    /// クローズ状態：通常動作中。
    case closed
    /// オープン状態：障害検知のため呼び出しをブロック。
    case open
    /// ハーフオープン状態：回復確認中。
    case halfOpen
}

/// サーキットブレーカーの設定。
public struct CircuitBreakerConfig: Sendable {
    /// オープン状態に遷移するまでの失敗回数閾値。
    public let failureThreshold: Int
    /// クローズ状態に戻るまでの成功回数閾値。
    public let successThreshold: Int
    /// オープン状態からハーフオープン状態に遷移するまでのタイムアウト（秒）。
    public let timeout: TimeInterval

    public init(
        failureThreshold: Int = 5,
        successThreshold: Int = 2,
        timeout: TimeInterval = 60.0
    ) {
        self.failureThreshold = failureThreshold
        self.successThreshold = successThreshold
        self.timeout = timeout
    }
}

/// サーキットブレーカー。障害の連鎖を防ぐ。
public actor CircuitBreaker {
    /// 現在の状態。
    public private(set) var state: CircuitBreakerState = .closed

    private let config: CircuitBreakerConfig
    private var failureCount: Int = 0
    private var successCount: Int = 0
    private var lastFailureTime: Date?

    public init(config: CircuitBreakerConfig = CircuitBreakerConfig()) {
        self.config = config
    }

    /// サーキットブレーカーがオープンかどうかを返す。
    public func isOpen() -> Bool {
        if state == .open {
            if let lastFailure = lastFailureTime,
               Date().timeIntervalSince(lastFailure) >= config.timeout {
                return false
            }
            return true
        }
        return false
    }

    /// 成功を記録する。
    public func recordSuccess() {
        switch state {
        case .halfOpen:
            successCount += 1
            if successCount >= config.successThreshold {
                state = .closed
                failureCount = 0
                successCount = 0
            }
        case .closed:
            failureCount = 0
        case .open:
            break
        }
    }

    /// 失敗を記録する。
    public func recordFailure() {
        lastFailureTime = Date()
        switch state {
        case .closed:
            failureCount += 1
            if failureCount >= config.failureThreshold {
                state = .open
                successCount = 0
            }
        case .halfOpen:
            state = .open
            successCount = 0
        case .open:
            break
        }
    }

    /// ハーフオープン状態への遷移を試みる（タイムアウト経過後）。
    public func tryTransitionToHalfOpen() {
        guard state == .open,
              let lastFailure = lastFailureTime,
              Date().timeIntervalSince(lastFailure) >= config.timeout else {
            return
        }
        state = .halfOpen
        successCount = 0
    }
}
