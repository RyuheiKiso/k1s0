/// イベントストア操作のエラー。
public enum EventStoreError: Error, Sendable {
    /// バージョン競合（楽観的ロック）。
    case versionConflict(expected: Int, actual: Int)
    /// ストリームが見つからなかった。
    case streamNotFound(String)
}

extension EventStoreError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .versionConflict(let expected, let actual):
            return "VERSION_CONFLICT: 期待バージョン \(expected)、実際のバージョン \(actual)"
        case .streamNotFound(let streamId):
            return "STREAM_NOT_FOUND: ストリーム '\(streamId)' が見つかりません"
        }
    }
}
