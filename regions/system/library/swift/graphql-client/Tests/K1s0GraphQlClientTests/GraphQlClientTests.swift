import Testing
@testable import K1s0GraphQlClient

@Suite("GraphQlClient Tests")
struct GraphQlClientTests {
    @Test("クエリを実行してレスポンスを取得できること")
    func testExecute() async throws {
        let client = InMemoryGraphQlClient()
        let response = GraphQlResponse<[String: String]>(data: ["name": "test"])
        await client.setResponse(response, forOperation: "GetUser")

        let query = GraphQlQuery(query: "{ user { name } }", operationName: "GetUser")
        let result: GraphQlResponse<[String: String]> = try await client.execute(query: query)
        #expect(result.data?["name"] == "test")
        #expect(result.hasErrors == false)
    }

    @Test("ミューテーションを実行してレスポンスを取得できること")
    func testExecuteMutation() async throws {
        let client = InMemoryGraphQlClient()
        let response = GraphQlResponse<[String: String]>(data: ["id": "123"])
        await client.setResponse(response, forOperation: "CreateUser")

        let mutation = GraphQlQuery(query: "mutation { createUser { id } }", operationName: "CreateUser")
        let result: GraphQlResponse<[String: String]> = try await client.executeMutation(mutation: mutation)
        #expect(result.data?["id"] == "123")
    }

    @Test("存在しないオペレーションでエラーになること")
    func testOperationNotFound() async throws {
        let client = InMemoryGraphQlClient()
        let query = GraphQlQuery(query: "{ user { name } }", operationName: "Unknown")

        do {
            let _: GraphQlResponse<[String: String]> = try await client.execute(query: query)
            #expect(Bool(false), "Should have thrown")
        } catch is GraphQlClientError {
            // expected
        }
    }

    @Test("オペレーション名なしでエラーになること")
    func testUnknownOperation() async throws {
        let client = InMemoryGraphQlClient()
        let query = GraphQlQuery(query: "{ user { name } }")

        do {
            let _: GraphQlResponse<[String: String]> = try await client.execute(query: query)
            #expect(Bool(false), "Should have thrown")
        } catch is GraphQlClientError {
            // expected
        }
    }

    @Test("エラーレスポンスを返すこと")
    func testErrorResponse() async throws {
        let client = InMemoryGraphQlClient()
        let error = GraphQlError(message: "Not found", locations: [ErrorLocation(line: 1, column: 5)], path: ["user"])
        let response = GraphQlResponse<[String: String]>(data: nil, errors: [error])
        await client.setResponse(response, forOperation: "GetUser")

        let query = GraphQlQuery(query: "{ user { name } }", operationName: "GetUser")
        let result: GraphQlResponse<[String: String]> = try await client.execute(query: query)
        #expect(result.hasErrors == true)
        #expect(result.errors?.first?.message == "Not found")
        #expect(result.errors?.first?.locations?.first?.line == 1)
        #expect(result.errors?.first?.path?.first == "user")
    }

    @Test("GraphQlQueryの変数が設定できること")
    func testQueryWithVariables() {
        let query = GraphQlQuery(
            query: "query($id: ID!) { user(id: $id) { name } }",
            variables: ["id": "123"],
            operationName: "GetUser"
        )
        #expect(query.query.contains("$id"))
        #expect(query.operationName == "GetUser")
    }

    @Test("GraphQlClientErrorの各バリアント")
    func testErrorVariants() {
        let err1 = GraphQlClientError.operationNotFound(name: "Test")
        if case .operationNotFound(let name) = err1 {
            #expect(name == "Test")
        }

        let err2 = GraphQlClientError.typeMismatch
        if case .typeMismatch = err2 {
            // expected
        } else {
            #expect(Bool(false), "Should be typeMismatch")
        }

        let err3 = GraphQlClientError.unknownOperation
        if case .unknownOperation = err3 {
            // expected
        } else {
            #expect(Bool(false), "Should be unknownOperation")
        }
    }
}
