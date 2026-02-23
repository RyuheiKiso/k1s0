public enum ValidationError: Error, Sendable {
    case invalidEmail(String)
    case invalidUUID(String)
    case invalidURL(String)
    case invalidTenantID(String)
}
