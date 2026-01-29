# @k1s0/realtime

k1s0 フロントエンド向けリアルタイム通信パッケージ（React）。

WebSocket・Server-Sent Events (SSE) クライアントと、グローバル接続管理 Provider を提供する。

## インストール

```bash
pnpm add @k1s0/realtime
```

## WebSocket

### useWebSocket

```tsx
import { useWebSocket } from '@k1s0/realtime';

function ChatRoom({ roomId }: { roomId: string }) {
  const { status, lastMessage, sendMessage, reconnectAttempt } =
    useWebSocket<ChatMessage>({
      url: `wss://api.example.com/ws/chat/${roomId}`,
      reconnect: {
        enabled: true,
        maxAttempts: 10,
        backoff: 'exponential',
        initialDelay: 1000,
        maxDelay: 30000,
      },
      heartbeat: {
        enabled: true,
        interval: 30000,
        timeout: 5000,
        message: JSON.stringify({ type: 'ping' }),
      },
      onMessage: (msg) => console.log('Received:', msg),
    });

  return (
    <div>
      <p>Status: {status}</p>
      <button onClick={() => sendMessage({ text: 'Hello' })}>Send</button>
    </div>
  );
}
```

### オプション

| プロパティ | 型 | デフォルト | 説明 |
|-----------|---|---------|------|
| `url` | `string` | 必須 | WebSocket URL |
| `protocols` | `string \| string[]` | - | サブプロトコル |
| `reconnect` | `Partial<ReconnectConfig>` | enabled | 再接続設定 |
| `heartbeat` | `Partial<HeartbeatConfig>` | disabled | ハートビート設定 |
| `autoConnect` | `boolean` | `true` | 自動接続 |
| `getAuthToken` | `() => string \| Promise<string>` | - | 認証トークン取得 |
| `serialize` | `(data: unknown) => string` | `JSON.stringify` | シリアライズ |
| `deserialize` | `(data: string) => T` | `JSON.parse` | デシリアライズ |

## Server-Sent Events

### useSSE

```tsx
import { useSSE } from '@k1s0/realtime';

function Dashboard() {
  const { status, lastEvent } = useSSE<DashboardUpdate>({
    url: '/api/dashboard/stream',
    withCredentials: true,
    eventHandlers: {
      metric: (data) => updateMetric(data),
      alert: (data) => showAlert(data),
    },
  });

  return <div>Status: {status}</div>;
}
```

## RealtimeProvider

グローバルな接続管理とオフラインキューを提供する。

```tsx
import { RealtimeProvider, useConnectionStatus, useOfflineQueue } from '@k1s0/realtime';

function App() {
  return (
    <RealtimeProvider
      config={{
        networkMonitor: { enabled: true },
        offlineQueue: { enabled: true, maxSize: 50, persistToStorage: true },
      }}
    >
      <Router />
    </RealtimeProvider>
  );
}

function StatusBar() {
  const { isOnline, connections } = useConnectionStatus();
  return <span>{isOnline ? 'Online' : 'Offline'}</span>;
}

function ChatInput() {
  const { queue, flush } = useOfflineQueue();
  // オフライン時はキューに蓄積、オンライン復帰時にフラッシュ
}
```

## API リファレンス

### Hooks

- `useWebSocket<T>(options)` - WebSocket 接続管理
- `useSSE<T>(options)` - SSE 接続管理
- `useConnectionStatus()` - グローバル接続状態
- `useOfflineQueue()` - オフラインキュー操作

### Components

- `RealtimeProvider` - グローバル Provider

### Types

- `ConnectionStatus` - `'connecting' | 'connected' | 'disconnecting' | 'disconnected'`
- `ReconnectConfig` - 再接続設定
- `HeartbeatConfig` - ハートビート設定
- `RealtimeConfig` - Provider 設定
- `ConnectionInfo` - 接続情報
- `OfflineQueueConfig` - オフラインキュー設定
