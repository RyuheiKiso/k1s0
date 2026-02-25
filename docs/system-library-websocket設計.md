# k1s0-websocket ライブラリ設計

## 概要

リアルタイム双方向通信のための WebSocket クライアントライブラリ。接続管理・自動再接続・Ping/Pong ハートビート・メッセージキューイングを統一インターフェースで提供する。notification-client の補完として、リアルタイム通知受信基盤を担う。全 Tier のサービス・クライアントから共通利用する。

**配置先**: `regions/system/library/rust/websocket/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `WsClient` | トレイト | WebSocket 接続・送受信インターフェース |
| `TungsteniteWsClient` | 構造体 | tokio-tungstenite による WebSocket 実装 |
| `WsConfig` | 構造体 | エンドポイント・再接続設定・ハートビート間隔・認証トークン |
| `WsMessage` | enum | `Text(String)`・`Binary(Vec<u8>)`・`Ping`・`Pong`・`Close` |
| `ConnectionState` | enum | `Disconnected`・`Connecting`・`Connected`・`Reconnecting` |
| `WsError` | enum | `Connection`・`Send`・`Receive`・`Timeout`・`MaxReconnectExceeded` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-websocket"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
futures-util = "0.3"

[dev-dependencies]
mockall = "0.13"
```

**モジュール構成**:

```
websocket/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # WsClient トレイト・TungsteniteWsClient
│   ├── config.rs       # WsConfig
│   ├── message.rs      # WsMessage
│   ├── state.rs        # ConnectionState
│   └── error.rs        # WsError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_websocket::{TungsteniteWsClient, WsConfig, WsMessage, WsClient};
use std::time::Duration;

// クライアントの構築
let config = WsConfig::new("wss://notification-server:8080/ws")
    .with_auth_token("Bearer <token>")
    .with_reconnect_interval(Duration::from_secs(5))
    .with_max_reconnect_attempts(10)
    .with_heartbeat_interval(Duration::from_secs(30));

let (client, mut receiver) = TungsteniteWsClient::connect(config).await?;

// メッセージ受信ループ
tokio::spawn(async move {
    while let Some(msg) = receiver.recv().await {
        match msg {
            WsMessage::Text(text) => {
                tracing::info!(message = %text, "メッセージ受信");
                // JSON デシリアライズして処理
            }
            WsMessage::Close => {
                tracing::info!("接続クローズ");
                break;
            }
            _ => {}
        }
    }
});

// メッセージ送信
client.send(WsMessage::Text(r#"{"type":"subscribe","channel":"notifications"}"#.to_string())).await?;

// 接続を閉じる
client.close().await?;
```

## Go 実装

**配置先**: `regions/system/library/go/websocket/`

```
websocket/
├── websocket.go        # WsClient インターフェース・GorillaWsClient
├── config.go           # WsConfig
├── message.go          # WsMessage
├── state.go            # ConnectionState
├── websocket_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/gorilla/websocket v1.5.3`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type WsClient interface {
    Send(ctx context.Context, msg WsMessage) error
    Receive(ctx context.Context) (<-chan WsMessage, error)
    Close() error
    State() ConnectionState
}

type WsConfig struct {
    Endpoint           string
    AuthToken          string
    ReconnectInterval  time.Duration
    MaxReconnects      int
    HeartbeatInterval  time.Duration
}

type MessageType int
const (
    MessageText   MessageType = iota
    MessageBinary
    MessagePing
    MessagePong
    MessageClose
)

type WsMessage struct {
    Type    MessageType
    Payload []byte
}

type ConnectionState int
const (
    StateDisconnected ConnectionState = iota
    StateConnecting
    StateConnected
    StateReconnecting
)

type GorillaWsClient struct{ /* ... */ }
func NewGorillaWsClient(cfg WsConfig) (*GorillaWsClient, <-chan WsMessage, error)
func (c *GorillaWsClient) Send(ctx context.Context, msg WsMessage) error
func (c *GorillaWsClient) Close() error
func (c *GorillaWsClient) State() ConnectionState
```

**使用例**:

```go
config := WsConfig{
    Endpoint:          "wss://notification-server:8080/ws",
    AuthToken:         "Bearer <token>",
    ReconnectInterval: 5 * time.Second,
    MaxReconnects:     10,
    HeartbeatInterval: 30 * time.Second,
}
client, msgCh, err := NewGorillaWsClient(config)
if err != nil {
    log.Fatal(err)
}
defer client.Close()

for msg := range msgCh {
    if msg.Type == MessageText {
        fmt.Printf("受信: %s\n", msg.Payload)
    }
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/websocket/`

```
websocket/
├── package.json        # "@k1s0/websocket", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # WsClient, WsConfig, WsMessage, ConnectionState, WsError
└── __tests__/
    └── websocket.test.ts
```

**主要 API**:

```typescript
export interface WsConfig {
  endpoint: string;
  authToken?: string;
  reconnectIntervalMs?: number;
  maxReconnectAttempts?: number;
  heartbeatIntervalMs?: number;
}

export type WsMessageType = 'text' | 'binary' | 'ping' | 'pong' | 'close';

export interface WsMessage {
  type: WsMessageType;
  payload?: string | ArrayBuffer;
}

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting';

export interface WsClient {
  send(message: WsMessage): Promise<void>;
  onMessage(handler: (message: WsMessage) => void): () => void;
  onStateChange(handler: (state: ConnectionState) => void): () => void;
  close(): Promise<void>;
  readonly state: ConnectionState;
}

export class NativeWsClient implements WsClient {
  constructor(config: WsConfig);
  send(message: WsMessage): Promise<void>;
  onMessage(handler: (message: WsMessage) => void): () => void;
  onStateChange(handler: (state: ConnectionState) => void): () => void;
  close(): Promise<void>;
  readonly state: ConnectionState;
}

export class WsError extends Error {
  constructor(message: string, public readonly code: 'CONNECTION' | 'SEND' | 'RECEIVE' | 'TIMEOUT' | 'MAX_RECONNECT_EXCEEDED');
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/websocket/`

```
websocket/
├── pubspec.yaml        # k1s0_websocket
├── analysis_options.yaml
├── lib/
│   ├── websocket.dart
│   └── src/
│       ├── client.dart     # WsClient abstract, NativeWsClient
│       ├── config.dart     # WsConfig
│       ├── message.dart    # WsMessage
│       ├── state.dart      # ConnectionState
│       └── error.dart      # WsError
└── test/
    └── websocket_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  web_socket_channel: ^3.0.0
```

**使用例**:

```dart
import 'package:k1s0_websocket/websocket.dart';

final config = WsConfig(
  endpoint: 'wss://notification-server:8080/ws',
  authToken: 'Bearer <token>',
  reconnectInterval: Duration(seconds: 5),
  maxReconnects: 10,
  heartbeatInterval: Duration(seconds: 30),
);

final client = NativeWsClient(config);
await client.connect();

// メッセージ受信
client.messages.listen((message) {
  if (message.type == WsMessageType.text) {
    print('受信: ${message.payload}');
  }
});

// メッセージ送信
await client.send(WsMessage(
  type: WsMessageType.text,
  payload: '{"type":"subscribe","channel":"notifications"}',
));

await client.close();
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/websocket/`

```
websocket/
├── src/
│   ├── WebSocket.csproj
│   ├── IWsClient.cs        # WebSocket クライアントインターフェース
│   ├── NativeWsClient.cs   # System.Net.WebSockets 実装
│   ├── WsConfig.cs         # WsConfig record
│   ├── WsMessage.cs        # WsMessage
│   └── WsException.cs      # 公開例外型
├── tests/
│   ├── WebSocket.Tests.csproj
│   └── Unit/
│       └── WsConfigTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| System.Net.WebSockets.Client | WebSocket クライアント |

**名前空間**: `K1s0.System.WebSocket`

**主要 API**:

```csharp
namespace K1s0.System.WebSocket;

public record WsConfig(
    string Endpoint,
    string? AuthToken = null,
    TimeSpan? ReconnectInterval = null,
    int MaxReconnectAttempts = 10,
    TimeSpan? HeartbeatInterval = null);

public enum WsMessageType { Text, Binary, Ping, Pong, Close }

public record WsMessage(WsMessageType Type, string? TextPayload = null, byte[]? BinaryPayload = null);

public enum ConnectionState { Disconnected, Connecting, Connected, Reconnecting }

public interface IWsClient : IAsyncDisposable
{
    Task SendAsync(WsMessage message, CancellationToken ct = default);
    IAsyncEnumerable<WsMessage> ReceiveAsync(CancellationToken ct = default);
    ConnectionState State { get; }
    event Action<ConnectionState>? StateChanged;
}

public sealed class NativeWsClient : IWsClient
{
    public NativeWsClient(WsConfig config);
    public Task ConnectAsync(CancellationToken ct = default);
    public Task SendAsync(WsMessage message, CancellationToken ct = default);
    public IAsyncEnumerable<WsMessage> ReceiveAsync(CancellationToken ct = default);
    public ConnectionState State { get; }
    public event Action<ConnectionState>? StateChanged;
    public ValueTask DisposeAsync();
}
```

**カバレッジ目標**: 90%以上
---

## Python 実装

**配置先**: `regions/system/library/python/websocket/`

### パッケージ構造

```
websocket/
├── pyproject.toml
├── src/
│   └── k1s0_websocket/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── client.py         # WsClient ABC・NativeWsClient
│       ├── config.py         # WsConfig dataclass
│       ├── message.py        # WsMessage dataclass・WsMessageType Enum
│       ├── state.py          # ConnectionState Enum
│       ├── exceptions.py     # WsError
│       └── py.typed
└── tests/
    ├── test_websocket.py
    └── test_reconnect.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `WsClient` | ABC | WebSocket 接続・送受信抽象基底クラス |
| `NativeWsClient` | class | websockets ライブラリによる WebSocket 実装 |
| `WsConfig` | dataclass | エンドポイント・再接続設定・ハートビート間隔 |
| `WsMessage` | dataclass | メッセージ種別・ペイロード |
| `ConnectionState` | Enum | 接続状態 |
| `WsError` | Exception | 基底エラークラス |

### 使用例

```python
import asyncio
from k1s0_websocket import NativeWsClient, WsConfig, WsMessage, WsMessageType

config = WsConfig(
    endpoint="wss://notification-server:8080/ws",
    auth_token="Bearer <token>",
    reconnect_interval_seconds=5,
    max_reconnect_attempts=10,
    heartbeat_interval_seconds=30,
)

async with NativeWsClient(config) as client:
    # メッセージ購読
    async for message in client.messages():
        if message.type == WsMessageType.TEXT:
            print(f"受信: {message.payload}")
        elif message.type == WsMessageType.CLOSE:
            break

    # メッセージ送信
    await client.send(WsMessage(
        type=WsMessageType.TEXT,
        payload='{"type":"subscribe","channel":"notifications"}',
    ))
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| websockets | >=13.0 | WebSocket クライアント |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_config_builder() {
        use std::time::Duration;
        let config = WsConfig::new("wss://example.com/ws")
            .with_reconnect_interval(Duration::from_secs(5))
            .with_max_reconnect_attempts(10);

        assert_eq!(config.max_reconnect_attempts, 10);
    }

    #[test]
    fn test_ws_message_types() {
        let text_msg = WsMessage::Text("hello".to_string());
        assert!(matches!(text_msg, WsMessage::Text(_)));

        let close_msg = WsMessage::Close;
        assert!(matches!(close_msg, WsMessage::Close));
    }
}
```

### 統合テスト

ローカル WebSocket サーバーを起動し、接続・送受信・再接続・ハートビートの各シナリオをカバー。

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestWsClient {}
    #[async_trait]
    impl WsClient for TestWsClient {
        async fn send(&self, message: WsMessage) -> Result<(), WsError>;
        fn state(&self) -> ConnectionState;
        async fn close(&self) -> Result<(), WsError>;
    }
}

#[tokio::test]
async fn test_notification_service_subscribes_on_connect() {
    let mut mock = MockTestWsClient::new();
    mock.expect_send()
        .withf(|msg| matches!(msg, WsMessage::Text(t) if t.contains("subscribe")))
        .once()
        .returning(|_| Ok(()));
    mock.expect_state()
        .returning(|| ConnectionState::Connected);

    let service = NotificationService::new(Arc::new(mock));
    service.subscribe("notifications").await.unwrap();
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-graphql-client設計](system-library-graphql-client設計.md) — GraphQL クライアント（サブスクリプション接続管理の上位ラッパー）
- [system-library-notification-client設計](system-library-notification-client設計.md) — 通知クライアント（WebSocket 経由のリアルタイム通知受信）
- [system-notification-server設計](system-notification-server設計.md) — 通知サーバー設計
- [メッセージング設計.md](メッセージング設計.md) — メッセージング設計
