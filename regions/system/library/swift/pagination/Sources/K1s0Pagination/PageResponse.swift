public struct PageResponse<T: Sendable>: Sendable {
    public let items: [T]
    public let total: UInt64
    public let page: UInt32
    public let perPage: UInt32
    public let totalPages: UInt32

    public init(items: [T], total: UInt64, page: UInt32, perPage: UInt32, totalPages: UInt32) {
        self.items = items
        self.total = total
        self.page = page
        self.perPage = perPage
        self.totalPages = totalPages
    }

    public static func create(items: [T], total: UInt64, request: PageRequest) -> PageResponse<T> {
        let totalPages = request.perPage > 0
            ? UInt32((total + UInt64(request.perPage) - 1) / UInt64(request.perPage))
            : 0
        return PageResponse(
            items: items,
            total: total,
            page: request.page,
            perPage: request.perPage,
            totalPages: totalPages
        )
    }
}
