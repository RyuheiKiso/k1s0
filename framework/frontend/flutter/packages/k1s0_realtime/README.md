# k1s0_realtime

k1s0 Flutter 向けリアルタイム通信パッケージ。

WebSocket・Server-Sent Events (SSE) クライアントと、グローバル接続管理マネージャーを提供する。

## インストール

```yaml
dependencies:
  k1s0_realtime:
    path: ../k1s0_realtime
```

## WebSocket

### K1s0WebSocket

```dart
import 'package:k1s0_realtime/k1s0_realtime.dart';

final socket = K1s0WebSocket(
  url: 'wss://api.example.com/ws/chat/room1',
  reconnectConfig: ReconnectConfig(
    enabled: true,
    maxAttempts: 10,
    backoffType: BackoffType.exponential,
  ),
  heartbeatConfig: HeartbeatConfig(
    enabled: true,
    interval: Duration(seconds: 30),
  ),
  getAuthToken: () => authService.token,
);

await socket.connect();

// メッセージの受信
socket.messages.listen((data) {
  print('Received: $data');
});

// 接続状態の監視
socket.status.listen((status) {
  print('Status: $status');
});

// メッセージの送信
socket.sendJson({'type': 'chat', 'text': 'Hello'});

// 切断
await socket.disconnect();
```

### Riverpod 統合

```dart
final chatSocketProvider = Provider.family<K1s0WebSocket, String>((ref, roomId) {
  final socket = K1s0WebSocket(
    url: 'wss://api.example.com/ws/chat/$roomId',
    getAuthToken: () => ref.read(authProvider).token,
  );
  ref.onDispose(() => socket.dispose());
  return socket;
});

final chatMessagesProvider = StreamProvider.family<dynamic, String>((ref, roomId) {
  final socket = ref.watch(chatSocketProvider(roomId));
  socket.connect();
  return socket.messages;
});
```

## Server-Sent Events

### K1s0SSE

```dart
final sse = K1s0SSE(
  url: 'https://api.example.com/notifications/stream',
  headers: {'Authorization': 'Bearer $token'},
);

await sse.connect();

sse.events.listen((event) {
  print('Event: ${event.eventType}, Data: ${event.data}');
  final parsed = event.parse((json) => Notification.fromJson(json));
});

await sse.disconnect();
```

## K1s0RealtimeManager

グローバルな接続管理とオフラインキューを提供する。

```dart
final manager = K1s0RealtimeManager(
  config: RealtimeConfig(
    networkMonitorEnabled: true,
    offlineQueue: OfflineQueueConfig(
      enabled: true,
      maxSize: 50,
      persistToStorage: true,
    ),
  ),
);

await manager.init();

// ネットワーク状態
manager.isOnline.listen((online) {
  print(online ? 'Online' : 'Offline');
});

// オフラインキュー
manager.queue('ws-chat', {'text': 'hello'});
final items = manager.flush<Map<String, dynamic>>('ws-chat');
```

### Riverpod Provider の例

```dart
final realtimeManagerProvider = Provider<K1s0RealtimeManager>((ref) {
  final manager = K1s0RealtimeManager();
  manager.init();
  ref.onDispose(() => manager.dispose());
  return manager;
});

final isOnlineProvider = StreamProvider<bool>((ref) {
  return ref.watch(realtimeManagerProvider).isOnline;
});
```

## API リファレンス

### クラス

- `K1s0WebSocket` - WebSocket クライアント
- `K1s0SSE` - SSE クライアント
- `K1s0RealtimeManager` - グローバル接続管理
- `OfflineQueue` - オフラインキュー
- `NetworkMonitor` - ネットワーク状態監視

### 型

- `ConnectionStatus` - 接続状態 enum
- `ReconnectConfig` - 再接続設定
- `HeartbeatConfig` - ハートビート設定
- `RealtimeConfig` - マネージャー設定
- `ConnectionInfo` - 接続情報
- `OfflineQueueConfig` - オフラインキュー設定
- `SSEEvent` - SSE イベント
- `BackoffType` - バックオフ戦略 enum
