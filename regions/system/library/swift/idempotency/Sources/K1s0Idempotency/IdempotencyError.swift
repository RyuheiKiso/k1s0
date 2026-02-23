/// 冪等性操作のエラー。
public enum IdempotencyError: Error, Sendable {
    /// 指定されたキーが見つからなかった。
    case notFound(String)
    /// 同じキーで既に処理中のリクエストが存在する。
    case alreadyProcessing(String)
    /// 保存操作に失敗した。
    case storageFailure(String)
    /// レコードの有効期限が切れた。
    case expired(String)
}

extension IdempotencyError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .notFound(let key):
            return "IDEMPOTENCY_NOT_FOUND: キー '\(key)' が見つかりません"
        case .alreadyProcessing(let key):
            return "IDEMPOTENCY_ALREADY_PROCESSING: キー '\(key)' は処理中です"
        case .storageFailure(let reason):
            return "IDEMPOTENCY_STORAGE_FAILURE: \(reason)"
        case .expired(let key):
            return "IDEMPOTENCY_EXPIRED: キー '\(key)' は期限切れです"
        }
    }
}
