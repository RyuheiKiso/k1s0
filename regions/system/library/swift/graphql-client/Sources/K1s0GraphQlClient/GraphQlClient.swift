import Foundation

public protocol GraphQlClient: Sendable {
    func execute<T: Decodable & Sendable>(query: GraphQlQuery) async throws -> GraphQlResponse<T>
    func executeMutation<T: Decodable & Sendable>(mutation: GraphQlQuery) async throws -> GraphQlResponse<T>
}

public actor InMemoryGraphQlClient: GraphQlClient {
    private var responses: [String: Any] = [:]

    public init() {}

    public func setResponse(_ response: Any, forOperation operationName: String) {
        responses[operationName] = response
    }

    public func execute<T: Decodable & Sendable>(query: GraphQlQuery) async throws -> GraphQlResponse<T> {
        try resolve(query)
    }

    public func executeMutation<T: Decodable & Sendable>(mutation: GraphQlQuery) async throws -> GraphQlResponse<T> {
        try resolve(mutation)
    }

    private func resolve<T: Decodable & Sendable>(_ query: GraphQlQuery) throws -> GraphQlResponse<T> {
        guard let name = query.operationName else {
            throw GraphQlClientError.unknownOperation
        }
        guard let stored = responses[name] else {
            throw GraphQlClientError.operationNotFound(name: name)
        }
        guard let typed = stored as? GraphQlResponse<T> else {
            throw GraphQlClientError.typeMismatch
        }
        return typed
    }
}
