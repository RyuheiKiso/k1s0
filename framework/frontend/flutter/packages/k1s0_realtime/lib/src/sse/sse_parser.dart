import '../types/sse_event.dart';

/// SSE プロトコルパーサー
///
/// Server-Sent Events のテキストストリームを [SSEEvent] に変換する。
class SSEParser {
  String? _eventType;
  String? _eventId;
  int? _retry;
  final StringBuffer _data = StringBuffer();
  bool _hasData = false;

  /// 1行を処理する
  ///
  /// イベントが完成した場合は [SSEEvent] を返す。
  /// まだ完成していない場合は null を返す。
  SSEEvent? parseLine(String line) {
    // 空行 = イベントのディスパッチ
    if (line.isEmpty) {
      if (!_hasData) {
        _reset();
        return null;
      }

      final event = SSEEvent(
        id: _eventId,
        eventType: _eventType ?? 'message',
        data: _data.toString().trimRight(),
        retry: _retry,
      );

      _reset();
      return event;
    }

    // コメント行
    if (line.startsWith(':')) {
      return null;
    }

    String field;
    String value;

    final colonIndex = line.indexOf(':');
    if (colonIndex == -1) {
      field = line;
      value = '';
    } else {
      field = line.substring(0, colonIndex);
      value = line.substring(colonIndex + 1);
      // コロンの直後のスペースを除去
      if (value.startsWith(' ')) {
        value = value.substring(1);
      }
    }

    switch (field) {
      case 'event':
        _eventType = value;
      case 'data':
        _hasData = true;
        if (_data.isNotEmpty) {
          _data.write('\n');
        }
        _data.write(value);
      case 'id':
        _eventId = value.isEmpty ? null : value;
      case 'retry':
        final ms = int.tryParse(value);
        if (ms != null) {
          _retry = ms;
        }
    }

    return null;
  }

  void _reset() {
    _eventType = null;
    _eventId = null;
    _retry = null;
    _data.clear();
    _hasData = false;
  }
}
