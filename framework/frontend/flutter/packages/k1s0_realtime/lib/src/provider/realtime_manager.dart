import 'dart:async';

import '../types/connection_info.dart';
import '../types/connection_status.dart';
import '../types/realtime_config.dart';
import '../utils/network_monitor.dart';
import 'offline_queue.dart';

/// k1s0 Realtime マネージャー
///
/// グローバルなリアルタイム接続管理、ネットワーク監視、オフラインキューを提供する。
class K1s0RealtimeManager {
  final RealtimeConfig config;
  late final NetworkMonitor _networkMonitor;
  late final OfflineQueue _offlineQueue;

  final StreamController<Map<String, ConnectionInfo>> _connectionsController =
      StreamController<Map<String, ConnectionInfo>>.broadcast();
  final Map<String, ConnectionInfo> _connections = {};

  K1s0RealtimeManager({this.config = const RealtimeConfig()}) {
    _networkMonitor = NetworkMonitor();
    _offlineQueue = OfflineQueue(config: config.offlineQueue);
  }

  /// ネットワークのオンライン状態ストリーム
  Stream<bool> get isOnline => _networkMonitor.onlineStream;

  /// 現在のオンライン状態
  bool get isCurrentlyOnline => _networkMonitor.isOnline;

  /// 登録済み接続のストリーム
  Stream<Map<String, ConnectionInfo>> get connections =>
      _connectionsController.stream;

  /// 現在の接続一覧
  Map<String, ConnectionInfo> get currentConnections =>
      Map.unmodifiable(_connections);

  /// オフラインキュー
  OfflineQueue get offlineQueue => _offlineQueue;

  /// 初期化する
  Future<void> init() async {
    if (config.networkMonitorEnabled) {
      await _networkMonitor.start();
    }
    await _offlineQueue.restore();
  }

  /// 接続を登録する
  void registerConnection(String id, ConnectionInfo info) {
    _connections[id] = info;
    _connectionsController.add(Map.unmodifiable(_connections));
  }

  /// 接続を更新する
  void updateConnection(String id, ConnectionStatus status,
      {int? reconnectAttempt}) {
    final existing = _connections[id];
    if (existing == null) return;

    _connections[id] = existing.copyWith(
      status: status,
      reconnectAttempt: reconnectAttempt,
      connectedAt:
          status == ConnectionStatus.connected ? DateTime.now() : null,
      disconnectedAt:
          status == ConnectionStatus.disconnected ? DateTime.now() : null,
    );
    _connectionsController.add(Map.unmodifiable(_connections));
  }

  /// 接続を解除する
  void unregisterConnection(String id) {
    _connections.remove(id);
    _connectionsController.add(Map.unmodifiable(_connections));
  }

  /// メッセージをキューに追加する
  void queue<T>(String connectionId, T item) {
    _offlineQueue.queue(connectionId, item);
  }

  /// キューをフラッシュする
  List<T> flush<T>(String connectionId) {
    return _offlineQueue.flush(connectionId);
  }

  /// リソースを解放する
  Future<void> dispose() async {
    await _networkMonitor.dispose();
    await _connectionsController.close();
  }
}
