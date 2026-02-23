/// スキーマレジストリエラー。
public enum SchemaRegistryError: Error, Sendable {
    case httpRequestFailed(String)
    case schemaNotFound(subject: String, version: Int?)
    case compatibilityViolation(subject: String, reason: String)
    case invalidSchema(String)
    case serialization(String)
    case unavailable(String)
}

extension SchemaRegistryError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .httpRequestFailed(let r): return "HTTP_REQUEST_FAILED: \(r)"
        case .schemaNotFound(let subject, let version):
            let vStr = version.map { ":\($0)" } ?? ""
            return "SCHEMA_NOT_FOUND: subject=\(subject)\(vStr)"
        case .compatibilityViolation(let subject, let reason):
            return "COMPATIBILITY_VIOLATION: subject=\(subject), reason=\(reason)"
        case .invalidSchema(let r): return "INVALID_SCHEMA: \(r)"
        case .serialization(let r): return "SERIALIZATION_ERROR: \(r)"
        case .unavailable(let r): return "UNAVAILABLE: \(r)"
        }
    }
}
