import Testing
@testable import K1s0SearchClient

@Suite("SearchClient Tests")
struct SearchClientTests {
    @Test("インデックスを作成しドキュメントを登録できること")
    func testIndexDocument() async throws {
        let client = InMemorySearchClient()
        try await client.createIndex(name: "products", mapping: IndexMapping())

        let doc = IndexDocument(id: "p-1", fields: ["name": "Rust Programming"])
        let result = try await client.indexDocument(index: "products", doc: doc)
        #expect(result.id == "p-1")
        #expect(result.version == 1)
    }

    @Test("バルクインデックスが成功すること")
    func testBulkIndex() async throws {
        let client = InMemorySearchClient()
        try await client.createIndex(name: "items", mapping: IndexMapping())

        let docs = [
            IndexDocument(id: "i-1", fields: ["name": "Item 1"]),
            IndexDocument(id: "i-2", fields: ["name": "Item 2"]),
        ]
        let result = try await client.bulkIndex(index: "items", docs: docs)
        #expect(result.successCount == 2)
        #expect(result.failedCount == 0)
        #expect(result.failures.isEmpty)
    }

    @Test("全文検索ができること")
    func testSearch() async throws {
        let client = InMemorySearchClient()
        try await client.createIndex(name: "products", mapping: IndexMapping())
        _ = try await client.indexDocument(index: "products", doc: IndexDocument(id: "p-1", fields: ["name": "Rust Programming"]))
        _ = try await client.indexDocument(index: "products", doc: IndexDocument(id: "p-2", fields: ["name": "Go Language"]))

        let query = SearchQuery(query: "Rust", facets: ["name"])
        let result = try await client.search(index: "products", query: query)
        #expect(result.total == 1)
        #expect(result.hits.count == 1)
        #expect(result.facets["name"] != nil)
    }

    @Test("存在しないインデックスで検索するとエラーになること")
    func testSearchIndexNotFound() async throws {
        let client = InMemorySearchClient()
        do {
            _ = try await client.search(index: "nonexistent", query: SearchQuery(query: "test"))
            #expect(Bool(false), "Should have thrown")
        } catch is SearchError {
            // expected
        }
    }

    @Test("ドキュメントを削除できること")
    func testDeleteDocument() async throws {
        let client = InMemorySearchClient()
        try await client.createIndex(name: "products", mapping: IndexMapping())
        _ = try await client.indexDocument(index: "products", doc: IndexDocument(id: "p-1", fields: ["name": "Test"]))

        try await client.deleteDocument(index: "products", id: "p-1")
        let count = await client.documentCount("products")
        #expect(count == 0)
    }

    @Test("空クエリで全件取得できること")
    func testEmptyQuery() async throws {
        let client = InMemorySearchClient()
        try await client.createIndex(name: "items", mapping: IndexMapping())
        _ = try await client.indexDocument(index: "items", doc: IndexDocument(id: "i-1", fields: ["name": "Item"]))

        let result = try await client.search(index: "items", query: SearchQuery(query: ""))
        #expect(result.total == 1)
    }

    @Test("IndexMappingのwithFieldでフィールドを追加できること")
    func testIndexMapping() {
        let mapping = IndexMapping()
            .withField("name", "text")
            .withField("price", "integer")
        #expect(mapping.fields.count == 2)
        #expect(mapping.fields["name"]?.fieldType == "text")
    }

    @Test("Filterファクトリメソッドが正しく動作すること")
    func testFilterFactories() {
        let eq = Filter.eq("status", "active")
        #expect(eq.operator == "eq")
        #expect(eq.field == "status")

        let range = Filter.range("price", 10, 100)
        #expect(range.operator == "range")
        #expect(range.valueTo != nil)
    }

    @Test("SearchErrorの各バリアント")
    func testSearchError() {
        let err1 = SearchError.indexNotFound(name: "test")
        if case .indexNotFound(let name) = err1 {
            #expect(name == "test")
        }

        let err2 = SearchError.timeout
        if case .timeout = err2 {
            // expected
        } else {
            #expect(Bool(false), "Should be timeout")
        }
    }
}
