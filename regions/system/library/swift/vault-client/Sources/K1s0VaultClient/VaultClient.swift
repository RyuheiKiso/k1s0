import Foundation

public struct Secret: Sendable {
    public let path: String
    public let data: [String: String]
    public let version: Int64
    public let createdAt: Date

    public init(path: String, data: [String: String], version: Int64, createdAt: Date) {
        self.path = path
        self.data = data
        self.version = version
        self.createdAt = createdAt
    }
}

public struct SecretRotatedEvent: Sendable {
    public let path: String
    public let version: Int64

    public init(path: String, version: Int64) {
        self.path = path
        self.version = version
    }
}

public struct VaultClientConfig: Sendable {
    public let serverUrl: String
    public let cacheTtl: Duration
    public let cacheMaxCapacity: Int

    public init(serverUrl: String, cacheTtl: Duration = .seconds(600), cacheMaxCapacity: Int = 500) {
        self.serverUrl = serverUrl
        self.cacheTtl = cacheTtl
        self.cacheMaxCapacity = cacheMaxCapacity
    }
}

public enum VaultError: Error, Sendable {
    case notFound(path: String)
    case permissionDenied(path: String)
    case serverError(message: String)
    case timeout
    case leaseExpired(path: String)
}

public protocol VaultClientProtocol: Sendable {
    func getSecret(path: String) async throws -> Secret
    func getSecretValue(path: String, key: String) async throws -> String
    func listSecrets(pathPrefix: String) async throws -> [String]
    func watchSecret(path: String) -> AsyncThrowingStream<SecretRotatedEvent, Error>
}

public actor InMemoryVaultClient: VaultClientProtocol {
    private var store: [String: Secret] = [:]
    private let config: VaultClientConfig

    public init(config: VaultClientConfig) {
        self.config = config
    }

    public func putSecret(_ secret: Secret) {
        store[secret.path] = secret
    }

    public func getSecret(path: String) async throws -> Secret {
        guard let secret = store[path] else {
            throw VaultError.notFound(path: path)
        }
        return secret
    }

    public func getSecretValue(path: String, key: String) async throws -> String {
        let secret = try await getSecret(path: path)
        guard let value = secret.data[key] else {
            throw VaultError.notFound(path: "\(path)/\(key)")
        }
        return value
    }

    public func listSecrets(pathPrefix: String) async throws -> [String] {
        store.keys.filter { $0.hasPrefix(pathPrefix) }
    }

    public func watchSecret(path: String) -> AsyncThrowingStream<SecretRotatedEvent, Error> {
        AsyncThrowingStream { continuation in
            continuation.finish()
        }
    }
}
