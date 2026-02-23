import Foundation

public struct TraceContext: Sendable {
    public let traceId: String
    public let parentId: String
    public let flags: UInt8

    public init(traceId: String, parentId: String, flags: UInt8 = 1) {
        self.traceId = traceId
        self.parentId = parentId
        self.flags = flags
    }

    public func toTraceparent() -> String {
        "00-\(traceId)-\(parentId)-\(String(format: "%02x", flags))"
    }

    public static func fromTraceparent(_ s: String) -> TraceContext? {
        let parts = s.split(separator: "-").map(String.init)
        guard parts.count == 4, parts[0] == "00" else { return nil }
        guard parts[1].count == 32, parts[2].count == 16, parts[3].count == 2 else { return nil }
        guard let flags = UInt8(parts[3], radix: 16) else { return nil }
        return TraceContext(traceId: parts[1], parentId: parts[2], flags: flags)
    }
}
