import 'dart:convert';

/// テスト用アサーションヘルパー。
class AssertionHelper {
  /// JSON 部分一致アサーション。
  static void assertJsonContains(dynamic actual, dynamic expected) {
    final actualObj = actual is String ? jsonDecode(actual) : actual;
    final expectedObj = expected is String ? jsonDecode(expected) : expected;
    if (!_jsonContains(actualObj, expectedObj)) {
      throw AssertionError(
        'JSON partial match failed.\n'
        'Actual: ${jsonEncode(actualObj)}\n'
        'Expected: ${jsonEncode(expectedObj)}',
      );
    }
  }

  /// イベント一覧に指定タイプのイベントが含まれるか検証する。
  static void assertEventEmitted(List<dynamic> events, String eventType) {
    final found = events.any((e) =>
        e is Map<String, dynamic> && e['type'] == eventType);
    if (!found) {
      throw AssertionError("Event '$eventType' not found in events");
    }
  }

  static bool _jsonContains(dynamic actual, dynamic expected) {
    if (expected is Map<String, dynamic>) {
      if (actual is! Map<String, dynamic>) return false;
      return expected.entries
          .every((e) => actual.containsKey(e.key) &&
              _jsonContains(actual[e.key], e.value));
    }
    if (expected is List) {
      if (actual is! List) return false;
      return expected.every((ev) => actual.any((av) => _jsonContains(av, ev)));
    }
    return actual == expected;
  }
}

/// アサーションエラー型。
class AssertionError implements Exception {
  final String message;
  const AssertionError(this.message);

  @override
  String toString() => message;
}
