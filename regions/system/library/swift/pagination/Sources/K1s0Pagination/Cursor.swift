import Foundation

public enum CursorError: Error, Sendable {
    case invalidCursor(String)
}

public func encodeCursor(_ id: String) -> String {
    Data(id.utf8).base64EncodedString()
}

public func decodeCursor(_ cursor: String) throws -> String {
    guard let data = Data(base64Encoded: cursor),
          let str = String(data: data, encoding: .utf8) else {
        throw CursorError.invalidCursor(cursor)
    }
    return str
}
