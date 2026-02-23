/// メッセージングエラー。
public enum MessagingError: Error, Sendable {
    case producerError(String)
    case consumerError(String)
    case serializationError(String)
    case deserializationError(String)
    case connectionError(String)
    case timeoutError(String)
    case publishError(String)
    case consumeError(String)
    case commitError(String)
}

extension MessagingError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .producerError(let r): return "PRODUCER_ERROR: \(r)"
        case .consumerError(let r): return "CONSUMER_ERROR: \(r)"
        case .serializationError(let r): return "SERIALIZATION_ERROR: \(r)"
        case .deserializationError(let r): return "DESERIALIZATION_ERROR: \(r)"
        case .connectionError(let r): return "CONNECTION_ERROR: \(r)"
        case .timeoutError(let r): return "TIMEOUT_ERROR: \(r)"
        case .publishError(let r): return "PUBLISH_ERROR: \(r)"
        case .consumeError(let r): return "CONSUME_ERROR: \(r)"
        case .commitError(let r): return "COMMIT_ERROR: \(r)"
        }
    }
}
