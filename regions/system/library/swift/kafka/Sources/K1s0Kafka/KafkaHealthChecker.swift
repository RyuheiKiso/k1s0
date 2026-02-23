/// Kafka ヘルス状態。
public enum KafkaHealthStatus: Sendable {
    case healthy
    case unhealthy(String)
}

/// Kafka ヘルスチェック。
public actor KafkaHealthChecker: Sendable {
    private let config: KafkaConfig

    public init(config: KafkaConfig) {
        self.config = config
    }

    /// 設定の妥当性を確認する（接続を試みる実装は省略）。
    public func checkConfig() throws {
        try config.validate()
    }

    /// ヘルスチェックを実行する。
    public func check() async -> KafkaHealthStatus {
        do {
            try config.validate()
            return .healthy
        } catch {
            return .unhealthy(String(describing: error))
        }
    }
}
