public enum ConnectionState: Sendable {
    case disconnected
    case connecting
    case connected
    case reconnecting
    case closing
}
