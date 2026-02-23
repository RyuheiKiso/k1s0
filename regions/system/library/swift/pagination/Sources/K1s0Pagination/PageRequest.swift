public struct PageRequest: Sendable {
    public let page: UInt32
    public let perPage: UInt32

    public init(page: UInt32, perPage: UInt32) {
        self.page = page
        self.perPage = perPage
    }
}
