import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

/// Dead Letter Queue HTTP REST クライアント。
public actor DlqClient: Sendable {
    private let endpoint: String
    private let session: URLSession

    public init(endpoint: String, session: URLSession = .shared) {
        self.endpoint = endpoint.hasSuffix("/") ? String(endpoint.dropLast()) : endpoint
        self.session = session
    }

    /// DLQ メッセージ一覧を取得する。
    public func listMessages(topic: String, page: Int = 1, pageSize: Int = 20) async throws -> ListDlqMessagesResponse {
        let url = try buildURL("/api/v1/dlq/\(topic)?page=\(page)&page_size=\(pageSize)")
        return try await get(url: url)
    }

    /// DLQ メッセージを取得する。
    public func getMessage(id: String) async throws -> DlqMessage {
        let url = try buildURL("/api/v1/dlq/messages/\(id)")
        return try await get(url: url)
    }

    /// DLQ メッセージを再処理する。
    public func retryMessage(id: String) async throws -> RetryDlqMessageResponse {
        let url = try buildURL("/api/v1/dlq/messages/\(id)/retry")
        return try await post(url: url)
    }

    /// DLQ メッセージを削除する。
    public func deleteMessage(id: String) async throws {
        let url = try buildURL("/api/v1/dlq/messages/\(id)")
        var request = URLRequest(url: url)
        request.httpMethod = "DELETE"
        let (_, response) = try await session.data(for: request)
        try validateResponse(response)
    }

    /// トピックの全メッセージを再処理する。
    public func retryAll(topic: String) async throws {
        let url = try buildURL("/api/v1/dlq/\(topic)/retry-all")
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        let (_, response) = try await session.data(for: request)
        try validateResponse(response)
    }

    // MARK: - Private

    private func buildURL(_ path: String) throws -> URL {
        guard let url = URL(string: endpoint + path) else {
            throw DlqError.httpRequestFailed("不正なURL: \(endpoint + path)")
        }
        return url
    }

    private func get<T: Decodable>(url: URL) async throws -> T {
        let (data, response) = try await session.data(from: url)
        try validateResponse(response)
        do {
            return try JSONDecoder().decode(T.self, from: data)
        } catch {
            throw DlqError.deserializeError(error.localizedDescription)
        }
    }

    private func post<T: Decodable>(url: URL) async throws -> T {
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        let (data, response) = try await session.data(for: request)
        try validateResponse(response)
        do {
            return try JSONDecoder().decode(T.self, from: data)
        } catch {
            throw DlqError.deserializeError(error.localizedDescription)
        }
    }

    private func validateResponse(_ response: URLResponse) throws {
        guard let httpResponse = response as? HTTPURLResponse else { return }
        guard (200..<300).contains(httpResponse.statusCode) else {
            throw DlqError.apiError(status: httpResponse.statusCode, message: "API エラー")
        }
    }
}
