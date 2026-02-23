import Foundation

public func validateEmail(_ email: String) throws {
    let pattern = #"^[^@\s]+@[^@\s]+\.[^@\s]+$"#
    guard email.range(of: pattern, options: .regularExpression) != nil else {
        throw ValidationError.invalidEmail(email)
    }
}

public func validateUUID(_ id: String) throws {
    guard UUID(uuidString: id) != nil else {
        throw ValidationError.invalidUUID(id)
    }
}

public func validateURL(_ rawURL: String) throws {
    guard let url = URL(string: rawURL),
          url.scheme == "http" || url.scheme == "https" else {
        throw ValidationError.invalidURL(rawURL)
    }
}

public func validateTenantID(_ tenantID: String) throws {
    let pattern = #"^[a-z0-9][a-z0-9\-]{1,61}[a-z0-9]$"#
    guard tenantID.range(of: pattern, options: .regularExpression) != nil else {
        throw ValidationError.invalidTenantID(tenantID)
    }
}
