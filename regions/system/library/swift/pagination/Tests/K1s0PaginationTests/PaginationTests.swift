import Testing
@testable import K1s0Pagination

@Suite("Pagination Tests")
struct PaginationTests {
    @Test("PageRequestが正しく初期化されること")
    func testPageRequest() {
        let req = PageRequest(page: 1, perPage: 10)
        #expect(req.page == 1)
        #expect(req.perPage == 10)
    }

    @Test("PageResponseが正しく計算されること")
    func testPageResponseCreate() {
        let req = PageRequest(page: 1, perPage: 10)
        let resp = PageResponse<String>.create(items: ["a", "b"], total: 25, request: req)
        #expect(resp.items.count == 2)
        #expect(resp.total == 25)
        #expect(resp.totalPages == 3)
    }

    @Test("perPageが0の場合totalPagesが0になること")
    func testZeroPerPage() {
        let req = PageRequest(page: 1, perPage: 0)
        let resp = PageResponse<String>.create(items: [], total: 10, request: req)
        #expect(resp.totalPages == 0)
    }

    @Test("totalが0の場合totalPagesが0になること")
    func testZeroTotal() {
        let req = PageRequest(page: 1, perPage: 10)
        let resp = PageResponse<String>.create(items: [], total: 0, request: req)
        #expect(resp.totalPages == 0)
    }

    @Test("カーソルのエンコードとデコードが正しく動作すること")
    func testCursorRoundTrip() throws {
        let sortKey = "2024-01-15"
        let id = "item-123"
        let encoded = encodeCursor(sortKey: sortKey, id: id)
        let (decodedSortKey, decodedId) = try decodeCursor(encoded)
        #expect(decodedSortKey == sortKey)
        #expect(decodedId == id)
    }

    @Test("無効なカーソルがエラーになること")
    func testInvalidCursor() {
        #expect(throws: CursorError.self) {
            try decodeCursor("!!!invalid-base64!!!")
        }
    }

    @Test("CursorRequestフィールド")
    func testCursorRequest() {
        let req = CursorRequest(cursor: "abc", limit: 20)
        #expect(req.cursor == "abc")
        #expect(req.limit == 20)
    }

    @Test("CursorMetaフィールド")
    func testCursorMeta() {
        let meta = CursorMeta(nextCursor: "next", hasMore: true)
        #expect(meta.nextCursor == "next")
        #expect(meta.hasMore == true)
    }

    @Test("PaginationMetaフィールド")
    func testPaginationMeta() {
        let meta = PaginationMeta(total: 100, page: 2, perPage: 10, totalPages: 10)
        #expect(meta.total == 100)
        #expect(meta.page == 2)
        #expect(meta.perPage == 10)
        #expect(meta.totalPages == 10)
    }

    @Test("validatePerPage有効値")
    func testValidatePerPageValid() throws {
        #expect(try validatePerPage(1) == 1)
        #expect(try validatePerPage(50) == 50)
        #expect(try validatePerPage(100) == 100)
    }

    @Test("validatePerPageゼロ")
    func testValidatePerPageZero() {
        #expect(throws: PerPageValidationError.self) {
            try validatePerPage(0)
        }
    }

    @Test("validatePerPage最大超過")
    func testValidatePerPageOverMax() {
        #expect(throws: PerPageValidationError.self) {
            try validatePerPage(101)
        }
    }
}
