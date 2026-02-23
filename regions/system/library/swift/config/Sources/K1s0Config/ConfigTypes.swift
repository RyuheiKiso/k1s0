import Foundation

/// アプリケーション設定。
public struct AppConfig: Codable, Sendable {
    public let name: String
    public let version: String
    public let tier: String
    public let environment: String
}

/// サーバー設定。
public struct ServerConfig: Codable, Sendable {
    public let host: String
    public let port: Int
    public let readTimeout: String?
    public let writeTimeout: String?
    public let shutdownTimeout: String?

    enum CodingKeys: String, CodingKey {
        case host, port
        case readTimeout = "read_timeout"
        case writeTimeout = "write_timeout"
        case shutdownTimeout = "shutdown_timeout"
    }
}

/// データベース設定。
public struct DatabaseConfig: Codable, Sendable {
    public let host: String
    public let port: Int
    public let name: String
    public let user: String
    public let password: String
    public let sslMode: String?

    enum CodingKeys: String, CodingKey {
        case host, port, name, user, password
        case sslMode = "ssl_mode"
    }
}

/// Kafka設定。
public struct KafkaConfig: Codable, Sendable {
    public let brokers: [String]
    public let consumerGroup: String
    public let securityProtocol: String

    enum CodingKeys: String, CodingKey {
        case brokers
        case consumerGroup = "consumer_group"
        case securityProtocol = "security_protocol"
    }
}

/// 認証設定。
public struct AuthConfig: Codable, Sendable {
    public struct JwtConfig: Codable, Sendable {
        public let issuer: String
        public let audience: String
        public let publicKeyPath: String?

        enum CodingKeys: String, CodingKey {
            case issuer, audience
            case publicKeyPath = "public_key_path"
        }
    }
    public let jwt: JwtConfig
}

/// ルート設定。
public struct Config: Codable, Sendable {
    public let app: AppConfig
    public let server: ServerConfig
    public let database: DatabaseConfig?
    public let kafka: KafkaConfig?
    public let auth: AuthConfig
}
