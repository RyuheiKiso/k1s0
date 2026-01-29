// Riverpod Provider 定義
//
// このファイルは Riverpod の Provider 定義を含む。
// flutter_riverpod を依存に追加した場合に使用する。
//
// 現在は Riverpod を optional dependency として扱うため、
// アプリ側で以下のように Provider を定義することを推奨する。
//
// ```dart
// final realtimeManagerProvider = Provider<K1s0RealtimeManager>((ref) {
//   final manager = K1s0RealtimeManager(config: RealtimeConfig());
//   manager.init();
//   ref.onDispose(() => manager.dispose());
//   return manager;
// });
//
// final isOnlineProvider = StreamProvider<bool>((ref) {
//   return ref.watch(realtimeManagerProvider).isOnline;
// });
//
// final connectionsProvider = StreamProvider<Map<String, ConnectionInfo>>((ref) {
//   return ref.watch(realtimeManagerProvider).connections;
// });
//
// final offlineQueueProvider = Provider<OfflineQueue>((ref) {
//   return ref.watch(realtimeManagerProvider).offlineQueue;
// });
// ```

export 'realtime_manager.dart';
export 'offline_queue.dart';
