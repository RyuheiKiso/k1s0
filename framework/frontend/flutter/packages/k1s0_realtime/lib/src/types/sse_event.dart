import 'dart:convert';

/// Server-Sent Event
class SSEEvent {
  /// イベント ID
  final String? id;

  /// イベントタイプ
  final String eventType;

  /// イベントデータ（生文字列）
  final String data;

  /// 再接続間隔（ms）
  final int? retry;

  const SSEEvent({
    this.id,
    required this.eventType,
    required this.data,
    this.retry,
  });

  /// JSON データをパースする
  T parse<T>(T Function(Map<String, dynamic>) fromJson) {
    return fromJson(jsonDecode(data) as Map<String, dynamic>);
  }

  /// データを JSON としてデコードする
  Map<String, dynamic> toJson() {
    return jsonDecode(data) as Map<String, dynamic>;
  }

  @override
  String toString() => 'SSEEvent(type: $eventType, id: $id, data: $data)';
}
