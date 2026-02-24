import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

/// HTTP GET リクエストでヘルスを確認する HealthCheck 実装。
public struct HttpHealthCheck: HealthCheck, Sendable {
    public let name: String
    private let url: URL
    private let timeoutInterval: TimeInterval

    public init(url: String, timeoutInterval: TimeInterval = 5.0, name: String? = nil) {
        guard let parsed = URL(string: url) else {
            fatalError("Invalid URL: \(url)")
        }
        self.url = parsed
        self.timeoutInterval = timeoutInterval
        self.name = name ?? "http"
    }

    public func check() async throws {
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.timeoutInterval = timeoutInterval

        let (_, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw HttpHealthCheckError.invalidResponse
        }

        guard (200..<300).contains(httpResponse.statusCode) else {
            throw HttpHealthCheckError.unhealthyStatus(
                url: url.absoluteString,
                statusCode: httpResponse.statusCode
            )
        }
    }
}

enum HttpHealthCheckError: Error, LocalizedError {
    case invalidResponse
    case unhealthyStatus(url: String, statusCode: Int)

    var errorDescription: String? {
        switch self {
        case .invalidResponse:
            return "HTTP check received non-HTTP response"
        case .unhealthyStatus(let url, let statusCode):
            return "HTTP \(url) returned status \(statusCode)"
        }
    }
}
