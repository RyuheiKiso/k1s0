import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

/// スキーマレジストリクライアントプロトコル。
public protocol SchemaRegistryClient: Sendable {
    func registerSchema(subject: String, schema: String, schemaType: SchemaType) async throws -> Int
    func getSchemaById(_ id: Int) async throws -> RegisteredSchema
    func getLatestSchema(subject: String) async throws -> RegisteredSchema
    func listSubjects() async throws -> [String]
    func checkCompatibility(subject: String, schema: String, schemaType: SchemaType) async throws -> Bool
    func healthCheck() async throws
}

/// URLSession を使用したスキーマレジストリクライアント。
public actor URLSessionSchemaRegistryClient: SchemaRegistryClient {
    private let config: SchemaRegistryConfig
    private let session: URLSession

    public init(config: SchemaRegistryConfig, session: URLSession = .shared) {
        self.config = config
        self.session = session
    }

    public func registerSchema(subject: String, schema: String, schemaType: SchemaType) async throws -> Int {
        let url = try buildURL("/subjects/\(subject)/versions")
        let body: [String: String] = ["schema": schema, "schemaType": schemaType.rawValue]
        let data = try JSONEncoder().encode(body)
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = data
        let (responseData, response) = try await session.data(for: request)
        try validateResponse(response)
        struct RegisterResponse: Decodable { let id: Int }
        let result = try JSONDecoder().decode(RegisterResponse.self, from: responseData)
        return result.id
    }

    public func getSchemaById(_ id: Int) async throws -> RegisteredSchema {
        let url = try buildURL("/schemas/ids/\(id)")
        return try await get(url: url)
    }

    public func getLatestSchema(subject: String) async throws -> RegisteredSchema {
        let url = try buildURL("/subjects/\(subject)/versions/latest")
        return try await get(url: url)
    }

    public func listSubjects() async throws -> [String] {
        let url = try buildURL("/subjects")
        return try await get(url: url)
    }

    public func checkCompatibility(subject: String, schema: String, schemaType: SchemaType) async throws -> Bool {
        let url = try buildURL("/compatibility/subjects/\(subject)/versions/latest")
        let body: [String: String] = ["schema": schema, "schemaType": schemaType.rawValue]
        let data = try JSONEncoder().encode(body)
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = data
        let (responseData, response) = try await session.data(for: request)
        try validateResponse(response)
        struct CompatResponse: Decodable { let isCompatible: Bool }
        let result = try JSONDecoder().decode(CompatResponse.self, from: responseData)
        return result.isCompatible
    }

    public func healthCheck() async throws {
        let url = try buildURL("/")
        let (_, response) = try await session.data(from: url)
        try validateResponse(response)
    }

    private func buildURL(_ path: String) throws -> URL {
        let base = config.url.hasSuffix("/") ? String(config.url.dropLast()) : config.url
        guard let url = URL(string: base + path) else {
            throw SchemaRegistryError.httpRequestFailed("不正なURL: \(base + path)")
        }
        return url
    }

    private func get<T: Decodable>(url: URL) async throws -> T {
        let (data, response) = try await session.data(from: url)
        try validateResponse(response)
        do {
            return try JSONDecoder().decode(T.self, from: data)
        } catch {
            throw SchemaRegistryError.serialization(error.localizedDescription)
        }
    }

    private func validateResponse(_ response: URLResponse) throws {
        guard let httpResponse = response as? HTTPURLResponse else { return }
        if httpResponse.statusCode == 404 {
            throw SchemaRegistryError.schemaNotFound(subject: "unknown", version: nil)
        }
        guard (200..<300).contains(httpResponse.statusCode) else {
            throw SchemaRegistryError.unavailable("HTTP \(httpResponse.statusCode)")
        }
    }
}
