public struct PageRequest: Sendable {
    public let page: UInt32
    public let perPage: UInt32

    public init(page: UInt32, perPage: UInt32) {
        self.page = page
        self.perPage = perPage
    }
}

public struct PaginationMeta: Sendable {
    public let total: UInt64
    public let page: UInt32
    public let perPage: UInt32
    public let totalPages: UInt32

    public init(total: UInt64, page: UInt32, perPage: UInt32, totalPages: UInt32) {
        self.total = total
        self.page = page
        self.perPage = perPage
        self.totalPages = totalPages
    }
}

public enum PerPageValidationError: Error, Sendable {
    case outOfRange(value: UInt32, min: UInt32, max: UInt32)
}

public let minPerPage: UInt32 = 1
public let maxPerPage: UInt32 = 100

public func validatePerPage(_ perPage: UInt32) throws -> UInt32 {
    guard perPage >= minPerPage, perPage <= maxPerPage else {
        throw PerPageValidationError.outOfRange(value: perPage, min: minPerPage, max: maxPerPage)
    }
    return perPage
}
