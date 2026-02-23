import Foundation

public struct WsConfig: Sendable {
    public let url: String
    public let reconnect: Bool
    public let maxReconnectAttempts: Int
    public let reconnectDelay: Duration
    public let pingInterval: Duration?

    public init(
        url: String = "ws://localhost",
        reconnect: Bool = true,
        maxReconnectAttempts: Int = 5,
        reconnectDelay: Duration = .seconds(1),
        pingInterval: Duration? = nil
    ) {
        self.url = url
        self.reconnect = reconnect
        self.maxReconnectAttempts = maxReconnectAttempts
        self.reconnectDelay = reconnectDelay
        self.pingInterval = pingInterval
    }

    public static let defaults = WsConfig()
}
