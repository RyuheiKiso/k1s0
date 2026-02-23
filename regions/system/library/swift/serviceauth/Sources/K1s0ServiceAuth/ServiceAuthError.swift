/// サービス認証エラー。
public enum ServiceAuthError: Error, Sendable {
    case tokenAcquisition(String)
    case tokenExpired
    case invalidToken(String)
    case spiffeValidationFailed(String)
    case oidcDiscovery(String)
    case http(String)
}

extension ServiceAuthError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .tokenAcquisition(let reason):
            return "TOKEN_ACQUISITION_ERROR: \(reason)"
        case .tokenExpired:
            return "TOKEN_EXPIRED"
        case .invalidToken(let reason):
            return "INVALID_TOKEN: \(reason)"
        case .spiffeValidationFailed(let reason):
            return "SPIFFE_VALIDATION_FAILED: \(reason)"
        case .oidcDiscovery(let reason):
            return "OIDC_DISCOVERY_ERROR: \(reason)"
        case .http(let reason):
            return "HTTP_ERROR: \(reason)"
        }
    }
}
