/// Kafka クライアント設定。
public struct KafkaConfig: Sendable {
    public let brokers: [String]
    public let securityProtocol: String
    public let consumerGroup: String?
    public let connectionTimeoutMs: UInt64
    public let requestTimeoutMs: UInt64
    public let maxMessageBytes: Int

    public init(
        brokers: [String],
        securityProtocol: String = "PLAINTEXT",
        consumerGroup: String? = nil,
        connectionTimeoutMs: UInt64 = 5000,
        requestTimeoutMs: UInt64 = 30000,
        maxMessageBytes: Int = 1_000_000
    ) {
        self.brokers = brokers
        self.securityProtocol = securityProtocol
        self.consumerGroup = consumerGroup
        self.connectionTimeoutMs = connectionTimeoutMs
        self.requestTimeoutMs = requestTimeoutMs
        self.maxMessageBytes = maxMessageBytes
    }

    /// ブートストラップサーバー文字列を返す。
    public var bootstrapServers: String {
        brokers.joined(separator: ",")
    }

    /// TLS を使用するか判定する。
    public var usesTLS: Bool {
        securityProtocol.contains("SSL") || securityProtocol.contains("TLS")
    }

    /// バリデーションを行う。
    public func validate() throws {
        guard !brokers.isEmpty else {
            throw KafkaError.configurationError("brokers は必須です")
        }
    }
}

/// Kafka トピック設定。
public struct TopicConfig: Sendable {
    /// 命名規則: `k1s0.{tier}.{domain}.{event-type}.{version}`
    public let name: String
    public let partitions: Int
    public let replicationFactor: Int
    public let retentionMs: Int64

    public init(
        name: String,
        partitions: Int = 3,
        replicationFactor: Int = 3,
        retentionMs: Int64 = 7 * 24 * 60 * 60 * 1000
    ) {
        self.name = name
        self.partitions = partitions
        self.replicationFactor = replicationFactor
        self.retentionMs = retentionMs
    }

    /// トピック名が命名規則に従うか検証する。
    public func validateName() -> Bool {
        let parts = name.split(separator: ".")
        return parts.count >= 4
    }
}
