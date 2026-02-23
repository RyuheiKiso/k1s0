import Foundation

/// リトライ動作を制御する設定。
public struct RetryConfig: Sendable {
    /// 最大試行回数。
    public let maxAttempts: Int
    /// 最初の遅延時間（秒）。
    public let initialDelay: TimeInterval
    /// 最大遅延時間（秒）。
    public let maxDelay: TimeInterval
    /// 遅延時間の乗数（指数バックオフ）。
    public let multiplier: Double
    /// ジッターを適用するか。
    public let jitter: Bool

    public init(
        maxAttempts: Int = 3,
        initialDelay: TimeInterval = 0.1,
        maxDelay: TimeInterval = 30.0,
        multiplier: Double = 2.0,
        jitter: Bool = false
    ) {
        self.maxAttempts = maxAttempts
        self.initialDelay = initialDelay
        self.maxDelay = maxDelay
        self.multiplier = multiplier
        self.jitter = jitter
    }

    /// デフォルトのリトライ設定。
    public static let `default` = RetryConfig()
}
