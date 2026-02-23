/// イベントストリームの識別子。
public struct StreamId: Sendable, Hashable, Equatable {
    /// ストリームIDの文字列値。
    public let value: String

    public init(_ value: String) {
        self.value = value
    }
}

extension StreamId: CustomStringConvertible {
    public var description: String { value }
}
