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
        let original = "item-123"
        let encoded = encodeCursor(original)
        let decoded = try decodeCursor(encoded)
        #expect(decoded == original)
    }

    @Test("無効なカーソルがエラーになること")
    func testInvalidCursor() {
        #expect(throws: CursorError.self) {
            try decodeCursor("!!!invalid-base64!!!")
        }
    }
}
