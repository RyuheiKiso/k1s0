import Foundation

/// テスト用アサーションヘルパー。
public struct AssertionHelper: Sendable {
    /// JSON 部分一致アサーション。
    public static func assertJsonContains(_ actual: String, _ expected: String) throws {
        guard let actualData = actual.data(using: .utf8),
              let expectedData = expected.data(using: .utf8),
              let actualObj = try? JSONSerialization.jsonObject(with: actualData),
              let expectedObj = try? JSONSerialization.jsonObject(with: expectedData) else {
            throw TestHelperError.assertionFailed(message: "Invalid JSON input")
        }
        guard jsonContains(actual: actualObj, expected: expectedObj) else {
            throw TestHelperError.assertionFailed(
                message: "JSON partial match failed.\nActual: \(actual)\nExpected: \(expected)")
        }
    }

    /// イベント一覧に指定タイプのイベントが含まれるか検証する。
    public static func assertEventEmitted(in events: [[String: Any]], type eventType: String) throws {
        let found = events.contains { e in
            (e["type"] as? String) == eventType
        }
        guard found else {
            throw TestHelperError.assertionFailed(
                message: "Event '\(eventType)' not found in events")
        }
    }

    private static func jsonContains(actual: Any, expected: Any) -> Bool {
        if let expectedDict = expected as? [String: Any],
           let actualDict = actual as? [String: Any] {
            return expectedDict.allSatisfy { key, value in
                guard let actualValue = actualDict[key] else { return false }
                return jsonContains(actual: actualValue, expected: value)
            }
        }
        if let expectedArray = expected as? [Any],
           let actualArray = actual as? [Any] {
            return expectedArray.allSatisfy { ev in
                actualArray.contains { av in jsonContains(actual: av, expected: ev) }
            }
        }
        return "\(actual)" == "\(expected)"
    }
}

/// テストヘルパーエラー型。
public enum TestHelperError: Error, Sendable {
    case assertionFailed(message: String)
    case jwtCreationFailed(underlying: Error)
}
