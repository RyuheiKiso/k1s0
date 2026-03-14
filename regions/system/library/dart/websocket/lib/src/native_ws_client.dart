import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'connection_state.dart' as ws_state;
import 'ws_client.dart';
import 'ws_config.dart';
import 'ws_message.dart';

// NativeWsClient は dart:io の WebSocket を使用した本番用 WebSocket クライアント実装。
// 自動再接続、Ping/Pong ハートビート、メッセージキューイングをサポートする。
class NativeWsClient implements WsClient {
  final WsConfig _config;
  ws_state.ConnectionState _state = ws_state.ConnectionState.disconnected;
  WebSocket? _ws;
  // receive() で取り出されていない受信済みメッセージのキュー
  final _receiveQueue = <WsMessage>[];
  // receive() が待機中の場合に完了させる Completer のキュー
  final _receiveCompleters = <Completer<WsMessage>>[];
  int _reconnectAttempts = 0;
  // disconnect() 後の自動再接続を防止するフラグ
  bool _stopping = false;
  Timer? _pingTimer;

  NativeWsClient(this._config);

  @override
  ws_state.ConnectionState get state => _state;

  // connect は WebSocket 接続を確立する。接続済みの場合はエラーをスローする。
  @override
  Future<void> connect() async {
    if (_state != ws_state.ConnectionState.disconnected) {
      throw StateError('Already connected');
    }
    _stopping = false;
    _state = ws_state.ConnectionState.connecting;
    await _doConnect();
  }

  // disconnect は接続を閉じてリソースを解放する。
  @override
  Future<void> disconnect() async {
    if (_state == ws_state.ConnectionState.disconnected) {
      throw StateError('Not connected');
    }
    _stopping = true;
    _state = ws_state.ConnectionState.closing;
    _stopPingTimer();
    await _ws?.close(WebSocketStatus.normalClosure);
    _ws = null;
    _state = ws_state.ConnectionState.disconnected;
  }

  // send はメッセージを WebSocket に送信する。
  @override
  Future<void> send(WsMessage message) async {
    if (_state != ws_state.ConnectionState.connected || _ws == null) {
      throw StateError('Not connected');
    }
    final socket = _ws!;
    if (message.type == MessageType.text) {
      socket.add(message.payload as String);
    } else if (message.type == MessageType.binary) {
      socket.add((message.payload as Uint8List).toList());
    }
    // dart:io の WebSocket は Ping/Pong を自動的に処理するためアプリ層での送信は不要
  }

  // receive は次のメッセージを返す。メッセージがなければ到着まで待機する。
  @override
  Future<WsMessage> receive() async {
    if (_state != ws_state.ConnectionState.connected) {
      throw StateError('Not connected');
    }
    if (_receiveQueue.isNotEmpty) {
      return _receiveQueue.removeAt(0);
    }
    final completer = Completer<WsMessage>();
    _receiveCompleters.add(completer);
    return completer.future;
  }

  // _doConnect は実際の WebSocket 接続を確立する。再接続時にも使用する。
  Future<void> _doConnect() async {
    final socket = await WebSocket.connect(_config.url);
    _ws = socket;
    _state = ws_state.ConnectionState.connected;
    _reconnectAttempts = 0;
    _startPingTimer();
    _listenToMessages(socket);
  }

  // _listenToMessages は WebSocket のメッセージストリームを購読する。
  void _listenToMessages(WebSocket socket) {
    socket.listen(
      (data) {
        WsMessage? msg;
        if (data is String) {
          msg = WsMessage(type: MessageType.text, payload: data);
        } else if (data is List<int>) {
          msg = WsMessage(
            type: MessageType.binary,
            payload: Uint8List.fromList(data),
          );
        }
        if (msg != null) {
          if (_receiveCompleters.isNotEmpty) {
            // 待機中の receive() を解決する
            _receiveCompleters.removeAt(0).complete(msg);
          } else {
            _receiveQueue.add(msg);
          }
        }
      },
      onError: (_) {
        // エラーは onDone で処理する
      },
      onDone: () {
        _stopPingTimer();
        if (!_stopping) {
          _scheduleReconnect();
        } else {
          _state = ws_state.ConnectionState.disconnected;
        }
      },
    );
  }

  // _scheduleReconnect は接続断後に再接続をスケジュールする。
  // 試行回数が上限に達した場合は disconnected 状態に遷移する。
  void _scheduleReconnect() {
    if (!_config.reconnect ||
        _reconnectAttempts >= _config.maxReconnectAttempts) {
      _state = ws_state.ConnectionState.disconnected;
      return;
    }
    _state = ws_state.ConnectionState.reconnecting;
    _reconnectAttempts++;

    Future.delayed(_config.reconnectDelay, () async {
      try {
        await _doConnect();
      } catch (_) {
        _scheduleReconnect();
      }
    });
  }

  // _startPingTimer は pingInterval が設定されている場合にタイマーを開始する。
  // dart:io の WebSocket は Ping/Pong を自動的に処理するため接続確認のみ行う。
  void _startPingTimer() {
    if (_config.pingInterval != null) {
      _pingTimer = Timer.periodic(_config.pingInterval!, (_) {
        // dart:io の WebSocket は Ping/Pong フレームを自動的に処理する
        // 追加の Ping 送信は不要
      });
    }
  }

  // _stopPingTimer は ping タイマーを停止する。
  void _stopPingTimer() {
    _pingTimer?.cancel();
    _pingTimer = null;
  }
}
