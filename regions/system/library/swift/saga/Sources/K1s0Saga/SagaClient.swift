import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

/// Saga REST クライアント。
public actor SagaClient: Sendable {
    private let endpoint: String
    private let session: URLSession

    public init(endpoint: String, session: URLSession = .shared) {
        self.endpoint = endpoint.hasSuffix("/") ? String(endpoint.dropLast()) : endpoint
        self.session = session
    }

    /// Saga を開始する。
    public func startSaga(_ request: StartSagaRequest) async throws -> StartSagaResponse {
        let url = try buildURL("/api/v1/sagas")
        let data = try JSONEncoder().encode(request)
        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = "POST"
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.httpBody = data
        return try await send(request: urlRequest)
    }

    /// Saga の状態を取得する。
    public func getSaga(id: String) async throws -> SagaState {
        let url = try buildURL("/api/v1/sagas/\(id)")
        let (data, response) = try await session.data(from: url)
        try validateResponse(response)
        return try decode(data)
    }

    /// Saga をキャンセルする。
    public func cancelSaga(id: String) async throws {
        let url = try buildURL("/api/v1/sagas/\(id)/cancel")
        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = "POST"
        let (_, response) = try await session.data(for: urlRequest)
        try validateResponse(response)
    }

    // MARK: - Private

    private func buildURL(_ path: String) throws -> URL {
        guard let url = URL(string: endpoint + path) else {
            throw SagaError.networkError("不正なURL: \(endpoint + path)")
        }
        return url
    }

    private func send<T: Decodable>(request: URLRequest) async throws -> T {
        let (data, response) = try await session.data(for: request)
        try validateResponse(response)
        return try decode(data)
    }

    private func decode<T: Decodable>(_ data: Data) throws -> T {
        do {
            return try JSONDecoder().decode(T.self, from: data)
        } catch {
            throw SagaError.deserializeError(error.localizedDescription)
        }
    }

    private func validateResponse(_ response: URLResponse) throws {
        guard let httpResponse = response as? HTTPURLResponse else { return }
        guard (200..<300).contains(httpResponse.statusCode) else {
            throw SagaError.apiError(statusCode: httpResponse.statusCode, message: "API エラー")
        }
    }
}
