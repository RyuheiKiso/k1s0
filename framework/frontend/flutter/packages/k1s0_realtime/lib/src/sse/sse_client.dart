import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;

import '../types/connection_status.dart';
import '../types/sse_event.dart';
import 'sse_parser.dart';

/// HTTP ベースの SSE クライアント
///
/// HTTP レスポンスのストリームを SSE イベントに変換する。
class SSEClientInternal {
  final http.Client _httpClient;
  final StreamController<ConnectionStatus> _statusController =
      StreamController<ConnectionStatus>.broadcast();
  final StreamController<SSEEvent> _eventController =
      StreamController<SSEEvent>.broadcast();
  StreamSubscription<String>? _subscription;
  ConnectionStatus _status = ConnectionStatus.disconnected;

  SSEClientInternal({http.Client? httpClient})
      : _httpClient = httpClient ?? http.Client();

  /// 接続状態のストリーム
  Stream<ConnectionStatus> get statusStream => _statusController.stream;

  /// イベントのストリーム
  Stream<SSEEvent> get eventStream => _eventController.stream;

  /// 現在の接続状態
  ConnectionStatus get status => _status;

  /// SSE 接続を開始する
  Future<void> connect(
    Uri uri, {
    Map<String, String>? headers,
    String? lastEventId,
  }) async {
    _setStatus(ConnectionStatus.connecting);

    try {
      final request = http.Request('GET', uri);
      request.headers['Accept'] = 'text/event-stream';
      request.headers['Cache-Control'] = 'no-cache';
      if (headers != null) {
        request.headers.addAll(headers);
      }
      if (lastEventId != null) {
        request.headers['Last-Event-ID'] = lastEventId;
      }

      final response = await _httpClient.send(request);

      if (response.statusCode != 200) {
        _setStatus(ConnectionStatus.disconnected);
        _eventController.addError(
          Exception('SSE connection failed with status ${response.statusCode}'),
        );
        return;
      }

      _setStatus(ConnectionStatus.connected);

      final parser = SSEParser();
      _subscription = response.stream
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .listen(
        (line) {
          final event = parser.parseLine(line);
          if (event != null) {
            _eventController.add(event);
          }
        },
        onError: (Object error) {
          _eventController.addError(error);
          _setStatus(ConnectionStatus.disconnected);
        },
        onDone: () {
          _setStatus(ConnectionStatus.disconnected);
        },
      );
    } catch (e) {
      _setStatus(ConnectionStatus.disconnected);
      _eventController.addError(e);
    }
  }

  /// 接続を切断する
  Future<void> disconnect() async {
    _setStatus(ConnectionStatus.disconnecting);
    await _subscription?.cancel();
    _subscription = null;
    _setStatus(ConnectionStatus.disconnected);
  }

  /// リソースを解放する
  Future<void> dispose() async {
    await disconnect();
    _httpClient.close();
    await _statusController.close();
    await _eventController.close();
  }

  void _setStatus(ConnectionStatus newStatus) {
    _status = newStatus;
    _statusController.add(newStatus);
  }
}
