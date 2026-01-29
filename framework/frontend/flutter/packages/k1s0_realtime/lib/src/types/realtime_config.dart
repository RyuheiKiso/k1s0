import 'heartbeat_config.dart';
import 'offline_queue_config.dart';
import 'reconnect_config.dart';

/// Realtime 設定
class RealtimeConfig {
  /// ネットワーク監視を有効にする
  final bool networkMonitorEnabled;

  /// オフラインキュー設定
  final OfflineQueueConfig offlineQueue;

  /// デフォルト再接続設定
  final ReconnectConfig defaultReconnect;

  /// デフォルトハートビート設定
  final HeartbeatConfig defaultHeartbeat;

  const RealtimeConfig({
    this.networkMonitorEnabled = true,
    this.offlineQueue = const OfflineQueueConfig(),
    this.defaultReconnect = const ReconnectConfig(),
    this.defaultHeartbeat = const HeartbeatConfig(),
  });
}
