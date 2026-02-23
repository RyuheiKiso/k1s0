/// Kafka エラー。
public enum KafkaError: Error, Sendable {
    case connectionFailed(String)
    case topicNotFound(String)
    case partitionError(String)
    case configurationError(String)
    case timeout(String)
}

extension KafkaError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .connectionFailed(let r): return "CONNECTION_FAILED: \(r)"
        case .topicNotFound(let r): return "TOPIC_NOT_FOUND: \(r)"
        case .partitionError(let r): return "PARTITION_ERROR: \(r)"
        case .configurationError(let r): return "CONFIGURATION_ERROR: \(r)"
        case .timeout(let r): return "TIMEOUT: \(r)"
        }
    }
}
