# k1s0-websocket ライブラリ設計

## 概要

リアルタイム双方向通信のための WebSocket クライアントライブラリ。接続管理・自動再接続・Ping/Pong ハートビート・メッセージキューイングを統一インターフェースで提供する。notification-client の補完として、リアルタイム通知受信基盤を担う。全 Tier のサービス・クライアントから共通利用する。

**配置先**: `regions/system/library/{rust,go,typescript,dart}/websocket/`

## 共通設計パターン

全4言語で統一された以下のパターンを採用している。

### WsClient インターフェース

| メソッド | シグネチャ概要 | 説明 |
|---------|-------------|------|
| `connect` | `() -> Result/Promise<void>` | WebSocket 接続を開始する |
| `disconnect` | `() -> Result/Promise<void>` | WebSocket 接続を切断する |
| `send` | `(message) -> Result/Promise<void>` | メッセージを送信する |
| `receive` | `() -> Result/Promise<message>` | メッセージを受信する |
| `state` | `() -> ConnectionState` | 現在の接続状態を返す（読み取り専用） |

### ConnectionState（5状態）

| 値 | 説明 |
|---|------|
| `Disconnected` | 未接続・切断済み |
| `Connecting` | 接続中 |
| `Connected` | 接続済み |
| `Reconnecting` | 再接続中 |
| `Closing` | 切断処理中 |

> **注**: 旧設計では4状態（`Closing` なし）だったが、実装では全言語で `Closing` が追加されている。

### WsConfig（接続設定）

| フィールド | 型 | デフォルト | 説明 |
|-----------|---|----------|------|
| `url` | `string` | `"ws://localhost"` | WebSocket エンドポイント URL |
| `reconnect` | `bool` | `true` | 自動再接続の有効/無効 |
| `maxReconnectAttempts` | `int` | `5` | 最大再接続試行回数 |
| `reconnectDelayMs` | `int` | `1000` | 再接続間隔（ミリ秒） |
| `pingIntervalMs` | `int?` | `None/null` | Ping 送信間隔（ミリ秒、オプション） |

> **注**: 旧設計に記載されていた `authToken`（認証トークン）フィールドは全言語の実装に存在しない。認証が必要な場合は接続時の URL パラメータやヘッダーで対応する想定。

### WsMessage（メッセージ型）

| バリアント | ペイロード | 説明 |
|-----------|----------|------|
| `Text` | `String` | テキストメッセージ |
| `Binary` | `byte[]` | バイナリメッセージ |
| `Ping` | `byte[]`（Rust）/ なし（Go/TS/Dart） | Ping フレーム |
| `Pong` | `byte[]`（Rust）/ なし（Go/TS/Dart） | Pong フレーム |
| `Close` | `CloseFrame?`（Rust）/ なし（Go/TS/Dart） | クローズフレーム |

### 実装クラス

全言語で `InMemoryWsClient`（テスト・開発用インメモリ実装）を提供している。本番用 WebSocket クライアント（`TungsteniteWsClient`、`GorillaWsClient` 等）は計画中。

## 公開 API 一覧

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `WsClient` | トレイト/インターフェース | WebSocket 接続・送受信インターフェース（connect/disconnect/send/receive/state） |
| `InMemoryWsClient` | 構造体/クラス | テスト・開発用インメモリ WebSocket クライアント |
| `WsConfig` | 構造体/クラス | URL・再接続設定・Ping 間隔 |
| `WsMessage` | enum/クラス | `Text`・`Binary`・`Ping`・`Pong`・`Close` |
| `CloseFrame` | 構造体（Rust のみ） | クローズフレームの `code` と `reason` |
| `ConnectionState` | enum | `Disconnected`・`Connecting`・`Connected`・`Reconnecting`・`Closing` |
| `WsError` | enum（Rust） | `ConnectionError`・`SendError`・`ReceiveError`・`NotConnected`・`AlreadyConnected`・`Closed` |

## Rust 実装

**配置先**: `regions/system/library/rust/websocket/`

**Cargo.toml**:

```toml
[package]
name = "k1s0-websocket"
version = "0.1.0"
edition = "2021"

[features]
default = []
mock = ["dep:mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["sync"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**モジュール構成**:

```
websocket/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # WsClient トレイト・InMemoryWsClient
│   ├── config.rs       # WsConfig（ビルダーパターン）
│   ├── message.rs      # WsMessage・CloseFrame
│   ├── state.rs        # ConnectionState（5状態）
│   └── error.rs        # WsError（6バリアント）
└── Cargo.toml
```

**公開 API（lib.rs）**:

```rust
pub use client::{InMemoryWsClient, WsClient};
pub use config::WsConfig;
pub use error::WsError;
pub use message::{CloseFrame, WsMessage};
pub use state::ConnectionState;
```

**WsClient トレイト**:

```rust
#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait WsClient: Send + Sync {
    async fn connect(&mut self) -> Result<(), WsError>;
    async fn disconnect(&mut self) -> Result<(), WsError>;
    async fn send(&self, message: WsMessage) -> Result<(), WsError>;
    async fn receive(&self) -> Result<WsMessage, WsError>;
    fn state(&self) -> ConnectionState;
}
```

**WsConfig（ビルダーパターン）**:

```rust
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub reconnect: bool,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
    pub ping_interval_ms: Option<u64>,
}

impl WsConfig {
    pub fn new(url: impl Into<String>) -> Self;
    pub fn reconnect(self, enabled: bool) -> Self;
    pub fn max_reconnect_attempts(self, max: u32) -> Self;
    pub fn reconnect_delay_ms(self, ms: u64) -> Self;
    pub fn ping_interval_ms(self, ms: u64) -> Self;
}
```

**WsMessage**:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}
```

**WsError**:

```rust
#[derive(Debug, Error)]
pub enum WsError {
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("send error: {0}")]
    SendError(String),
    #[error("receive error: {0}")]
    ReceiveError(String),
    #[error("not connected")]
    NotConnected,
    #[error("already connected")]
    AlreadyConnected,
    #[error("closed: {0}")]
    Closed(String),
}
```

**使用例**:

```rust
use k1s0_websocket::{InMemoryWsClient, WsClient, WsConfig, WsMessage};

// 設定の構築
let config = WsConfig::new("wss://notification-server:8080/ws")
    .reconnect(true)
    .max_reconnect_attempts(10)
    .reconnect_delay_ms(5000)
    .ping_interval_ms(30000);

// InMemoryWsClient（テスト用）
let mut client = InMemoryWsClient::new();
client.connect().await?;

// メッセージ送信
client.send(WsMessage::Text(r#"{"type":"subscribe","channel":"notifications"}"#.to_string())).await?;

// メッセージ受信（テスト時は push_receive で注入）
client.push_receive(WsMessage::Text("hello".to_string())).await;
let msg = client.receive().await?;

// 接続状態確認
assert_eq!(client.state(), ConnectionState::Connected);

// 切断
client.disconnect().await?;
```

> **注**: 本番用 WebSocket クライアント（`TungsteniteWsClient` 等、`tokio-tungstenite` ベース）は計画中。現在の実装は `InMemoryWsClient`（テスト・開発用）のみ。`mock` feature を有効にすると `mockall::automock` による `MockWsClient` が利用可能。

## Go 実装

**配置先**: `regions/system/library/go/websocket/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/stretchr/testify v1.10.0`（テスト用）

**主要インターフェース**:

```go
// WsClient はWebSocketクライアントのインターフェース。
type WsClient interface {
    Connect(ctx context.Context) error
    Disconnect(ctx context.Context) error
    Send(ctx context.Context, msg Message) error
    Receive(ctx context.Context) (Message, error)
    State() ConnectionState
}
```

**型定義**:

```go
type MessageType int
const (
    MessageText   MessageType = iota
    MessageBinary
    MessagePing
    MessagePong
    MessageClose
)

type Message struct {
    Type    MessageType
    Payload []byte
}

type ConnectionState int
const (
    StateDisconnected  ConnectionState = iota
    StateConnecting
    StateConnected
    StateReconnecting
    StateClosing
)

type Config struct {
    URL                  string
    Reconnect            bool
    MaxReconnectAttempts int
    ReconnectDelayMs     int64
    PingIntervalMs       *int64
}

// DefaultConfig はデフォルト設定を返す。
func DefaultConfig() Config
```

**InMemoryWsClient**:

```go
type InMemoryWsClient struct{ /* ... */ }

func NewInMemoryWsClient() *InMemoryWsClient
func (c *InMemoryWsClient) Connect(ctx context.Context) error
func (c *InMemoryWsClient) Disconnect(ctx context.Context) error
func (c *InMemoryWsClient) Send(ctx context.Context, msg Message) error
func (c *InMemoryWsClient) Receive(ctx context.Context) (Message, error)
func (c *InMemoryWsClient) State() ConnectionState

// テスト用ヘルパー
func (c *InMemoryWsClient) InjectMessage(msg Message)
func (c *InMemoryWsClient) SentMessages() []Message
```

**使用例**:

```go
import websocket "github.com/k1s0-platform/system-library-go-websocket"

// デフォルト設定
config := websocket.DefaultConfig()
config.URL = "wss://notification-server:8080/ws"
config.MaxReconnectAttempts = 10

// InMemoryWsClient（テスト用）
client := websocket.NewInMemoryWsClient()
ctx := context.Background()

err := client.Connect(ctx)
if err != nil {
    log.Fatal(err)
}
defer client.Disconnect(ctx)

// メッセージ送信
msg := websocket.Message{
    Type:    websocket.MessageText,
    Payload: []byte(`{"type":"subscribe","channel":"notifications"}`),
}
err = client.Send(ctx, msg)

// メッセージ受信（テスト時は InjectMessage で注入）
client.InjectMessage(websocket.Message{
    Type:    websocket.MessageText,
    Payload: []byte("hello"),
})
received, err := client.Receive(ctx)
fmt.Printf("受信: %s\n", received.Payload)
```

> **注**: 本番用 WebSocket クライアント（`GorillaWsClient` 等、`gorilla/websocket` ベース）は計画中。現在の実装は `InMemoryWsClient`（テスト・開発用）のみ。

## TypeScript 実装

**配置先**: `regions/system/library/typescript/websocket/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**モジュール構成**:

```
websocket/src/
├── index.ts   # 公開 API（再エクスポート）
├── types.ts   # WsConfig・WsMessage・ConnectionState・MessageType・defaultConfig
└── client.ts  # WsClient インターフェース・InMemoryWsClient
```

**主要 API**:

```typescript
// types.ts
export type MessageType = 'text' | 'binary' | 'ping' | 'pong' | 'close';

export interface WsMessage {
  type: MessageType;
  payload: string | Uint8Array;
}

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting' | 'closing';

export interface WsConfig {
  url: string;
  reconnect: boolean;
  maxReconnectAttempts: number;
  reconnectDelayMs: number;
  pingIntervalMs?: number;
}

export function defaultConfig(): WsConfig;

// client.ts
export interface WsClient {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  send(message: WsMessage): Promise<void>;
  receive(): Promise<WsMessage>;
  readonly state: ConnectionState;
}

export class InMemoryWsClient implements WsClient {
  get state(): ConnectionState;
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  send(message: WsMessage): Promise<void>;
  receive(): Promise<WsMessage>;

  // テスト用ヘルパー
  injectMessage(msg: WsMessage): void;
  getSentMessages(): WsMessage[];
}
```

**使用例**:

```typescript
import { InMemoryWsClient, defaultConfig } from 'k1s0-websocket';
import type { WsMessage } from 'k1s0-websocket';

// 設定
const config = {
  ...defaultConfig(),
  url: 'wss://notification-server:8080/ws',
  maxReconnectAttempts: 10,
  reconnectDelayMs: 5000,
};

// InMemoryWsClient（テスト用）
const client = new InMemoryWsClient();
await client.connect();

// メッセージ送信
await client.send({
  type: 'text',
  payload: '{"type":"subscribe","channel":"notifications"}',
});

// メッセージ受信（テスト時は injectMessage で注入）
client.injectMessage({ type: 'text', payload: 'hello' });
const msg = await client.receive();
console.log('受信:', msg.payload);

// 接続状態確認
console.log('状態:', client.state); // 'connected'

// 切断
await client.disconnect();
```

> **注**: 本番用 WebSocket クライアント（`NativeWsClient` 等、ブラウザ/Node.js WebSocket API ベース）は計画中。現在の実装は `InMemoryWsClient`（テスト・開発用）のみ。`receive()` は受信バッファが空の場合 Promise で待機する（resolver キュー方式）。

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/websocket/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**モジュール構成**:

```
websocket/lib/
├── websocket.dart          # ライブラリ定義（再エクスポート）
└── src/
    ├── ws_client.dart       # WsClient 抽象クラス・InMemoryWsClient
    ├── ws_config.dart       # WsConfig
    ├── ws_message.dart      # WsMessage・MessageType
    └── connection_state.dart # ConnectionState（5状態）
```

**主要 API**:

```dart
// connection_state.dart
enum ConnectionState {
  disconnected,
  connecting,
  connected,
  reconnecting,
  closing,
}

// ws_message.dart
enum MessageType { text, binary, ping, pong, close }

class WsMessage {
  final MessageType type;
  final Object payload;

  const WsMessage({required this.type, required this.payload});
  String get textPayload => payload as String;
  Uint8List get binaryPayload => payload as Uint8List;
}

// ws_config.dart
class WsConfig {
  final String url;
  final bool reconnect;                  // default: true
  final int maxReconnectAttempts;        // default: 5
  final Duration reconnectDelay;         // default: 1秒
  final Duration? pingInterval;          // default: null

  const WsConfig({required this.url, ...});
  static WsConfig get defaults;
}

// ws_client.dart
abstract class WsClient {
  Future<void> connect();
  Future<void> disconnect();
  Future<void> send(WsMessage message);
  Future<WsMessage> receive();
  ConnectionState get state;
}

class InMemoryWsClient implements WsClient {
  // テスト用ヘルパー
  List<WsMessage> get sentMessages;
  void injectMessage(WsMessage msg);
}
```

**使用例**:

```dart
import 'package:k1s0_websocket/websocket.dart';

// 設定
final config = WsConfig(
  url: 'wss://notification-server:8080/ws',
  reconnect: true,
  maxReconnectAttempts: 10,
  reconnectDelay: Duration(seconds: 5),
  pingInterval: Duration(seconds: 30),
);

// InMemoryWsClient（テスト用）
final client = InMemoryWsClient();
await client.connect();

// メッセージ送信
await client.send(WsMessage(
  type: MessageType.text,
  payload: '{"type":"subscribe","channel":"notifications"}',
));

// メッセージ受信（テスト時は injectMessage で注入）
client.injectMessage(WsMessage(
  type: MessageType.text,
  payload: 'hello',
));
final msg = await client.receive();
print('受信: ${msg.textPayload}');

// 接続状態確認
print('状態: ${client.state}'); // ConnectionState.connected

// 切断
await client.disconnect();
```

> **注**: 本番用 WebSocket クライアント（`NativeWsClient` 等、`web_socket_channel` ベース）は計画中。現在の実装は `InMemoryWsClient`（テスト・開発用）のみ。Dart 版では `reconnectDelay` は `Duration` 型（他言語のミリ秒整数とは異なる）。

**カバレッジ目標**: 90%以上

## 設計書と実装の差分まとめ

旧設計書から実装への主な変更点を以下に整理する。

| 項目 | 旧設計書 | 実装（全4言語共通） |
|-----|---------|-----------------|
| 主要メソッド | `send` / `close` + チャネル受信 | `connect` / `disconnect` / `send` / `receive` / `state` |
| ConnectionState | 4状態（Closing なし） | 5状態（`Closing` 追加） |
| Config フィールド名 | `endpoint`, `authToken`, `heartbeatInterval` 等 | `url`, `reconnect`, `maxReconnectAttempts`, `reconnectDelayMs`, `pingIntervalMs` |
| AuthToken | Config に含む | Config に存在しない |
| 実装クラス | `TungsteniteWsClient` / `GorillaWsClient` / `NativeWsClient` | `InMemoryWsClient`（全言語共通、テスト用） |
| WsError（Rust） | `Connection` / `Send` / `Receive` / `Timeout` / `MaxReconnectExceeded` | `ConnectionError` / `SendError` / `ReceiveError` / `NotConnected` / `AlreadyConnected` / `Closed` |
| mock サポート（Rust） | `mockall` dev-dependency | `mock` feature flag（optional dependency） |
| WsMessage::Close（Rust） | 引数なし | `Option<CloseFrame>` 付き |
| WsMessage::Ping/Pong（Rust） | 引数なし | `Vec<u8>` ペイロード付き |
| Go 型名 | `WsConfig` / `WsMessage` | `Config` / `Message`（プレフィックスなし） |

## テスト戦略

### ユニットテスト

全言語で `InMemoryWsClient` を使用したユニットテストを実装済み。

**Rust（`cargo test --lib`）**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut client = InMemoryWsClient::new();
        assert_eq!(client.state(), ConnectionState::Disconnected);

        client.connect().await.unwrap();
        assert_eq!(client.state(), ConnectionState::Connected);

        client.disconnect().await.unwrap();
        assert_eq!(client.state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_send_receive() {
        let mut client = InMemoryWsClient::new();
        client.connect().await.unwrap();

        client.push_receive(WsMessage::Text("hello".to_string())).await;
        let msg = client.receive().await.unwrap();
        assert_eq!(msg, WsMessage::Text("hello".to_string()));

        client.send(WsMessage::Text("world".to_string())).await.unwrap();
        let sent = client.pop_sent().await.unwrap();
        assert_eq!(sent, WsMessage::Text("world".to_string()));
    }
}
```

**Go（`go test ./...`）**:

```go
func TestSendReceive(t *testing.T) {
    c := websocket.NewInMemoryWsClient()
    ctx := context.Background()
    _ = c.Connect(ctx)

    sendMsg := websocket.Message{Type: websocket.MessageText, Payload: []byte("hello")}
    err := c.Send(ctx, sendMsg)
    require.NoError(t, err)

    sent := c.SentMessages()
    require.Len(t, sent, 1)
    assert.Equal(t, []byte("hello"), sent[0].Payload)
}
```

### モックテスト（Rust）

`mock` feature を有効にすると `mockall::automock` マクロにより `MockWsClient` が自動生成される。

```rust
// Cargo.toml で mock feature を有効化
// k1s0-websocket = { path = "...", features = ["mock"] }

use k1s0_websocket::MockWsClient;

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockWsClient::new();
    mock.expect_send()
        .withf(|msg| matches!(msg, WsMessage::Text(t) if t.contains("subscribe")))
        .once()
        .returning(|_| Ok(()));
    mock.expect_state()
        .returning(|| ConnectionState::Connected);
}
```

### 統合テスト

ローカル WebSocket サーバーを起動し、接続・送受信・再接続・ハートビートの各シナリオをカバー。

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) --- ライブラリ一覧・テスト方針
- [system-library-graphql-client設計](../client-sdk/graphql-client.md) --- GraphQL クライアント（サブスクリプション接続管理の上位ラッパー）
- [system-library-notification-client設計](notification-client.md) --- 通知クライアント（WebSocket 経由のリアルタイム通知受信）
- [system-notification-server設計](../../servers/notification/server.md) --- notification-server 設計
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) --- メッセージング設計
