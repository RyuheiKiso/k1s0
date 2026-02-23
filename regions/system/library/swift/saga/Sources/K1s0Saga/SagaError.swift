/// Saga エラー。
public enum SagaError: Error, Sendable {
    case networkError(String)
    case deserializeError(String)
    case apiError(statusCode: Int, message: String)
}

extension SagaError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .networkError(let r): return "NETWORK_ERROR: \(r)"
        case .deserializeError(let r): return "DESERIALIZE_ERROR: \(r)"
        case .apiError(let code, let msg): return "API_ERROR(\(code)): \(msg)"
        }
    }
}
