import Foundation

/// DLQ エラー。
public enum DlqError: Error, Sendable {
    case httpRequestFailed(String)
    case apiError(status: Int, message: String)
    case deserializeError(String)
}

extension DlqError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .httpRequestFailed(let r): return "HTTP_REQUEST_FAILED: \(r)"
        case .apiError(let status, let msg): return "API_ERROR(\(status)): \(msg)"
        case .deserializeError(let r): return "DESERIALIZE_ERROR: \(r)"
        }
    }
}
