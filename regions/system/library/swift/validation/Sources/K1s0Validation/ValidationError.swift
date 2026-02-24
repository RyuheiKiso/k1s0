public enum ValidationError: Error, Sendable {
    case invalidEmail(String)
    case invalidUUID(String)
    case invalidURL(String)
    case invalidTenantID(String)
    case invalidPage(Int)
    case invalidPerPage(Int)
    case invalidDateRange(String)

    /// Returns the error code string for this validation error.
    public var code: String {
        switch self {
        case .invalidEmail: return "INVALID_EMAIL"
        case .invalidUUID: return "INVALID_UUID"
        case .invalidURL: return "INVALID_URL"
        case .invalidTenantID: return "INVALID_TENANT_ID"
        case .invalidPage: return "INVALID_PAGE"
        case .invalidPerPage: return "INVALID_PER_PAGE"
        case .invalidDateRange: return "INVALID_DATE_RANGE"
        }
    }
}

/// A collection of `ValidationError` instances.
public struct ValidationErrors: Sendable {
    private var errors: [ValidationError] = []

    public init() {}

    public func hasErrors() -> Bool {
        !errors.isEmpty
    }

    public func getErrors() -> [ValidationError] {
        errors
    }

    public mutating func add(_ error: ValidationError) {
        errors.append(error)
    }
}
