import Foundation

public enum CursorError: Error, Sendable {
    case invalidCursor(String)
}

public struct CursorRequest: Sendable {
    public let cursor: String?
    public let limit: UInt32

    public init(cursor: String? = nil, limit: UInt32) {
        self.cursor = cursor
        self.limit = limit
    }
}

public struct CursorMeta: Sendable {
    public let nextCursor: String?
    public let hasMore: Bool

    public init(nextCursor: String? = nil, hasMore: Bool) {
        self.nextCursor = nextCursor
        self.hasMore = hasMore
    }
}

private let cursorSeparator: Character = "|"

public func encodeCursor(sortKey: String, id: String) -> String {
    Data("\(sortKey)\(cursorSeparator)\(id)".utf8).base64EncodedString()
}

public func decodeCursor(_ cursor: String) throws -> (sortKey: String, id: String) {
    guard let data = Data(base64Encoded: cursor),
          let str = String(data: data, encoding: .utf8) else {
        throw CursorError.invalidCursor(cursor)
    }
    guard let idx = str.firstIndex(of: cursorSeparator) else {
        throw CursorError.invalidCursor("missing separator")
    }
    let sortKey = String(str[str.startIndex..<idx])
    let id = String(str[str.index(after: idx)...])
    return (sortKey, id)
}
