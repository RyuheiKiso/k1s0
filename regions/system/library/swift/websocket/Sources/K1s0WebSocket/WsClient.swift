import Foundation

public enum WsError: Error, Sendable {
    case notConnected
    case noMessagesAvailable
    case alreadyConnected
    case connectionFailed(reason: String)
}

public protocol WsClient: Sendable {
    func connect() async throws
    func disconnect() async throws
    func send(_ message: WsMessage) async throws
    func receive() async throws -> WsMessage
    var state: ConnectionState { get async }
}

public actor InMemoryWsClient: WsClient {
    private var _state: ConnectionState = .disconnected
    private var receiveBuffer: [WsMessage] = []
    private var sentMessages: [WsMessage] = []
    private let config: WsConfig

    public init(config: WsConfig = .defaults) {
        self.config = config
    }

    public var state: ConnectionState {
        _state
    }

    public func connect() async throws {
        guard _state == .disconnected || _state == .reconnecting else {
            throw WsError.alreadyConnected
        }
        _state = .connecting
        _state = .connected
    }

    public func disconnect() async throws {
        _state = .closing
        _state = .disconnected
        receiveBuffer.removeAll()
    }

    public func send(_ message: WsMessage) async throws {
        guard _state == .connected else {
            throw WsError.notConnected
        }
        sentMessages.append(message)
    }

    public func receive() async throws -> WsMessage {
        guard _state == .connected else {
            throw WsError.notConnected
        }
        guard !receiveBuffer.isEmpty else {
            throw WsError.noMessagesAvailable
        }
        return receiveBuffer.removeFirst()
    }

    public func injectMessage(_ msg: WsMessage) {
        receiveBuffer.append(msg)
    }

    public func getSentMessages() -> [WsMessage] {
        sentMessages
    }
}
