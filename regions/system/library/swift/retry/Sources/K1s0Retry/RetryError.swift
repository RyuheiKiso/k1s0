/// リトライ操作のエラー。
public enum RetryError: Error, Sendable {
    /// 最大試行回数に達した。
    case maxAttemptsReached(Int)
    /// 操作がキャンセルされた。
    case cancelled
}

extension RetryError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .maxAttemptsReached(let attempts):
            return "MAX_ATTEMPTS_REACHED: \(attempts)回試行後に失敗"
        case .cancelled:
            return "RETRY_CANCELLED: 操作がキャンセルされました"
        }
    }
}
