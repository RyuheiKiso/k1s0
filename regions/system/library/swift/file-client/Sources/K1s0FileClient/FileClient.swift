import Foundation

public struct FileMetadata: Sendable {
    public let path: String
    public let sizeBytes: Int64
    public let contentType: String
    public let etag: String
    public let lastModified: Date
    public let tags: [String: String]

    public init(
        path: String,
        sizeBytes: Int64,
        contentType: String,
        etag: String,
        lastModified: Date,
        tags: [String: String]
    ) {
        self.path = path
        self.sizeBytes = sizeBytes
        self.contentType = contentType
        self.etag = etag
        self.lastModified = lastModified
        self.tags = tags
    }
}

public struct PresignedUrl: Sendable {
    public let url: String
    public let method: String
    public let expiresAt: Date
    public let headers: [String: String]

    public init(url: String, method: String, expiresAt: Date, headers: [String: String]) {
        self.url = url
        self.method = method
        self.expiresAt = expiresAt
        self.headers = headers
    }
}

public enum FileClientError: Error, Sendable {
    case connectionFailed(underlying: String)
    case notFound(path: String)
    case unauthorized
    case quotaExceeded
    case invalidConfig(reason: String)
}

public protocol FileClient: Sendable {
    func generateUploadUrl(
        path: String,
        contentType: String,
        expiresIn: Duration
    ) async throws -> PresignedUrl

    func generateDownloadUrl(path: String, expiresIn: Duration) async throws -> PresignedUrl
    func delete(path: String) async throws
    func getMetadata(path: String) async throws -> FileMetadata
    func list(prefix: String) async throws -> [FileMetadata]
    func copy(src: String, dst: String) async throws
}

public actor InMemoryFileClient: FileClient {
    private var files: [String: FileMetadata] = [:]

    public init() {}

    public func storedFiles() -> [FileMetadata] {
        Array(files.values)
    }

    public func generateUploadUrl(
        path: String,
        contentType: String,
        expiresIn: Duration
    ) async throws -> PresignedUrl {
        files[path] = FileMetadata(
            path: path,
            sizeBytes: 0,
            contentType: contentType,
            etag: "",
            lastModified: Date(),
            tags: [:]
        )
        let seconds = Double(expiresIn.components.seconds)
        return PresignedUrl(
            url: "https://storage.example.com/upload/\(path)",
            method: "PUT",
            expiresAt: Date().addingTimeInterval(seconds),
            headers: [:]
        )
    }

    public func generateDownloadUrl(path: String, expiresIn: Duration) async throws -> PresignedUrl {
        guard files[path] != nil else {
            throw FileClientError.notFound(path: path)
        }
        let seconds = Double(expiresIn.components.seconds)
        return PresignedUrl(
            url: "https://storage.example.com/download/\(path)",
            method: "GET",
            expiresAt: Date().addingTimeInterval(seconds),
            headers: [:]
        )
    }

    public func delete(path: String) async throws {
        guard files.removeValue(forKey: path) != nil else {
            throw FileClientError.notFound(path: path)
        }
    }

    public func getMetadata(path: String) async throws -> FileMetadata {
        guard let meta = files[path] else {
            throw FileClientError.notFound(path: path)
        }
        return meta
    }

    public func list(prefix: String) async throws -> [FileMetadata] {
        files.values.filter { $0.path.hasPrefix(prefix) }
    }

    public func copy(src: String, dst: String) async throws {
        guard let source = files[src] else {
            throw FileClientError.notFound(path: src)
        }
        files[dst] = FileMetadata(
            path: dst,
            sizeBytes: source.sizeBytes,
            contentType: source.contentType,
            etag: source.etag,
            lastModified: source.lastModified,
            tags: source.tags
        )
    }
}
