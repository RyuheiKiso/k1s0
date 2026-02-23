import Foundation

/// 相関ID（UUID v4ベース）。
public struct CorrelationId: Hashable, Equatable, Sendable, Codable, CustomStringConvertible {
    private let value: String

    /// 新しい相関IDを生成する。
    public init() {
        self.value = UUID().uuidString.lowercased()
    }

    /// 文字列から相関IDを生成する。
    public init(string: String) {
        self.value = string
    }

    public var asString: String { value }

    public var description: String { value }
}
