/// キャッシュ操作のエラー。
public enum CacheError: Error, Sendable {
    /// 指定されたキーが見つからなかった。
    case notFound(String)
    /// ストレージ操作に失敗した。
    case storageFailure(String)
}

extension CacheError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .notFound(let key):
            return "CACHE_NOT_FOUND: キー '\(key)' が見つかりません"
        case .storageFailure(let reason):
            return "CACHE_STORAGE_FAILURE: \(reason)"
        }
    }
}
