import Foundation

/// トレースID（32文字16進数）。
public struct TraceId: Hashable, Equatable, Sendable, Codable, CustomStringConvertible {
    private let value: String

    /// 新しいトレースIDを生成する。
    public init() {
        self.value = UUID().uuidString.replacingOccurrences(of: "-", with: "").lowercased()
    }

    private init(validated: String) {
        self.value = validated
    }

    /// 文字列からトレースIDを生成する（32文字16進数でなければ nil）。
    public static func from(string: String) -> TraceId? {
        let lowercased = string.lowercased()
        guard lowercased.count == 32,
              lowercased.allSatisfy({ $0.isHexDigit }) else {
            return nil
        }
        return TraceId(validated: lowercased)
    }

    public var asString: String { value }

    public var description: String { value }
}
