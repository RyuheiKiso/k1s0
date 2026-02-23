/// 冪等性レコードの処理状態。
public enum IdempotencyStatus: String, Sendable, Equatable {
    /// 処理待ち。
    case pending
    /// 処理中。
    case processing
    /// 完了。
    case completed
    /// 失敗。
    case failed
}
