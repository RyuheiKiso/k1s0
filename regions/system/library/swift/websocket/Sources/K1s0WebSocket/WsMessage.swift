import Foundation

public enum WsMessageType: Sendable {
    case text
    case binary
    case ping
    case pong
    case close
}

public struct WsMessage: Sendable {
    public let type: WsMessageType
    public let payload: Data

    public init(type: WsMessageType, payload: Data = Data()) {
        self.type = type
        self.payload = payload
    }

    public static func text(_ s: String) -> WsMessage {
        WsMessage(type: .text, payload: Data(s.utf8))
    }

    public static func binary(_ data: Data) -> WsMessage {
        WsMessage(type: .binary, payload: data)
    }

    public var textValue: String? {
        if type == .text {
            return String(data: payload, encoding: .utf8)
        }
        return nil
    }
}
