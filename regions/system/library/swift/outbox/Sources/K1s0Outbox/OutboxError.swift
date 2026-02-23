/// アウトボックスエラー。
public enum OutboxError: Error, Sendable {
    case storeError(String)
    case publishError(String)
    case serializationError(String)
    case notFound(String)
}

extension OutboxError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .storeError(let r): return "STORE_ERROR: \(r)"
        case .publishError(let r): return "PUBLISH_ERROR: \(r)"
        case .serializationError(let r): return "SERIALIZATION_ERROR: \(r)"
        case .notFound(let r): return "NOT_FOUND: \(r)"
        }
    }
}
