// transport_negotiation.go — k1s0 tier1 Library Go: Transport Negotiation Runtime
// docs/03_概要設計/02_tier1設計方針/02_Library.md §Companion 役割 B に準拠する。
// tier1 Server 系 Transport Adapter Layer と対をなすクライアント実装を提供する。
// 8 adapter（sse_paired / long_poll / webhook / websocket / web_transport / messaging_bridge
//            / grpc_web / connect_rpc）の chosen_transport capability negotiation を担う。
// Rust frontend/transport_negotiation.rs / TypeScript frontend/transport_negotiation.ts /
// C# Frontend/TransportNegotiation.cs と 4 言語等価強度を保つ。
// OSS の transport 型（http.Client / gorilla/websocket 等）を公開 API に一切露出しない。

// パッケージ名: frontend（tier1 Library の frontend 向け transport negotiation API を提供する）
package frontend

import (
	// context: context.Context（非同期操作 / キャンセル制御に使用する）
	"context"
	// errors: エラー生成に使用する
	"errors"
	// fmt: エラーメッセージのフォーマットに使用する
	"fmt"
)

// ---- Transport adapter 種別定義 ----

// TransportKind は tier1 Server が選択可能な 8 transport adapter 種別を宣言する型。
// Gateway の chosen_transport フィールドと 1:1 対応する Library 独自語彙とする。
// Rust TransportKind / TypeScript TransportKind / C# TransportKind と 4 言語等価強度を保つ。
type TransportKind string

const (
	// TransportSsePaired: 既定。SSE フレーミング + 送信用 unary POST の組合せ
	// resume_token は SSE id: フィールド / Last-Event-ID ヘッダーで透過再接続する
	TransportSsePaired TransportKind = "sse_paired"
	// TransportLongPoll: cursor 付き short poll の組合せ（SSE 非対応環境向け）
	TransportLongPoll TransportKind = "long_poll"
	// TransportWebhook: レガシー側が HTTP サーバーとして受信する opt-in adapter
	TransportWebhook TransportKind = "webhook"
	// TransportWebSocket: WebSocket ベース（HTTP Upgrade 対応環境向け）
	TransportWebSocket TransportKind = "websocket"
	// TransportWebTransport: QUIC / WebTransport クライアント向け（v1 では opt-in adapter 扱い）
	TransportWebTransport TransportKind = "web_transport"
	// TransportMessagingBridge: Kafka / AMQP の REST Proxy 越しに bidi メッセージを搬送する
	TransportMessagingBridge TransportKind = "messaging_bridge"
	// TransportGrpcWeb: gRPC-Web プロトコル（HTTP/1.1 対応環境で gRPC を使用する場合）
	TransportGrpcWeb TransportKind = "grpc_web"
	// TransportConnectRpc: ConnectRPC プロトコル（gRPC / gRPC-Web との相互運用性が高い）
	TransportConnectRpc TransportKind = "connect_rpc"
)

// ---- クライアント Capability 宣言 ----

// ClientCapabilities は Companion が起動時に Gateway に送出する capability 宣言を定義する型。
// 利用可能な adapter 一覧 / TLS バージョン / inbound 可否 / max message size 等を宣言する。
// Open RPC の client_capabilities 形式と互換性を保つ。
// Rust ClientCapabilities / C# ClientCapabilities と 4 言語等価強度を保つ。
type ClientCapabilities struct {
	// AvailableTransports: クライアントが利用可能な transport adapter 一覧
	// Gateway はこの一覧から chosen_transport を選択する
	AvailableTransports []TransportKind
	// TLSMinVersion: クライアントがサポートする TLS 最低バージョン（例: "TLSv1.2" / "TLSv1.3"）
	TLSMinVersion string
	// InboundCapable: Webhook adapter を受信できるかどうか（HTTP サーバーとして動作可能な場合 true）
	InboundCapable bool
	// MaxMessageSizeBytes: 1 メッセージの最大サイズ（バイト）
	// 0 = 制限なし（実装側の OS / stack の制限に従う）
	MaxMessageSizeBytes uint64
	// ResumeTokenSupport: resume_token による再接続をサポートするかどうか
	// SsePaired / WebSocket / WebTransport adapter で有効化する
	ResumeTokenSupport bool
}

// DefaultClientCapabilities は frontend 向け安全なデフォルト ClientCapabilities を返す。
// SSE + LongPoll + WebSocket の 3 adapter をデフォルトで利用可能とする。
func DefaultClientCapabilities() ClientCapabilities {
	// frontend 向け安全なデフォルト設定を返す
	return ClientCapabilities{
		// SSE + LongPoll + WebSocket の 3 adapter をデフォルトで利用可能とする
		AvailableTransports: []TransportKind{
			TransportSsePaired,
			TransportLongPoll,
			TransportWebSocket,
		},
		// TLS 1.2 をデフォルト最低バージョンとする（TLS 1.3 推奨だが互換性のため 1.2 を下限とする）
		TLSMinVersion: "TLSv1.2",
		// デフォルトでは inbound を受け入れない（Webhook opt-in が必要）
		InboundCapable: false,
		// デフォルト最大メッセージサイズ: 4MB（大半の業務 RPC に十分な値）
		MaxMessageSizeBytes: 4 * 1024 * 1024,
		// デフォルトで resume_token をサポートする（SsePaired の id: フィールドを使用する）
		ResumeTokenSupport: true,
	}
}

// ---- 双方向チャンネル抽象 ----

// BidiMessage はアプリ側が送受信する双方向メッセージを宣言する型。
// transport 種別を意識しない統一型（プロトコルバッファのバイト列を運ぶ）。
// Rust BidiMessage / TypeScript BidiMessage / C# BidiMessage と 4 言語等価強度を保つ。
type BidiMessage struct {
	// Payload: メッセージボディ（protobuf バイト列）
	Payload []byte
	// SequenceID: メッセージ順序番号（resume_token 透過化に使用する）
	// 送信側が単調増加で付与し、受信側は順序検証に使用する
	SequenceID uint64
	// ResumeToken: セッション再接続トークン（transport 切替後の継続に使用する）
	// SsePaired の場合は SSE id: フィールド、WebSocket の場合は拡張ヘッダーで搬送する
	ResumeToken string
}

// ---- BidiChannel interface ----

// BidiChannel は双方向 RPC セッションのアプリ向け抽象 interface を宣言する。
// アプリ側は transport 種別を一切意識せず Send / Recv の API のみを使用する。
// 再接続は BidiChannel 実装内部で透過的に処理される（アプリに漏れない）。
// Rust BidiChannel / TypeScript BidiChannel / C# IBidiChannel と 4 言語等価強度を保つ。
type BidiChannel interface {
	// Send はアプリ側からメッセージを送信する。
	// transport が切り替わっても呼び出し元は継続できる（透過再接続）。
	Send(ctx context.Context, payload []byte) error

	// Recv は受信メッセージを取得する。
	// セッション終了時は nil, nil を返す（io.EOF 相当）。
	// transport 切替による中断は内部でハンドルする（アプリに漏れない）。
	Recv(ctx context.Context) (*BidiMessage, error)

	// Close はセッションをグレースフルに終了する（送信未完了メッセージをフラッシュする）。
	Close(ctx context.Context) error

	// ChosenTransport は現在アクティブな transport 種別を返す（デバッグ / ログ用）。
	ChosenTransport() TransportKind

	// Capabilities は起動時に送出したクライアント capability 宣言を返す（デバッグ用）。
	Capabilities() ClientCapabilities
}

// ---- TransportNegotiationClient interface ----

// TransportNegotiationClient は transport negotiation を行って BidiChannel を開設する facade interface。
// Gateway との negotiation フローを抽象化し、アプリ側は transport 選択結果を受け取るだけでよい。
// Rust TransportNegotiationClient / TypeScript TransportNegotiationClient /
// C# ITransportNegotiationClient と 4 言語等価強度を保つ。
type TransportNegotiationClient interface {
	// Negotiate は Gateway との capability exchange を行い、最適な BidiChannel を返す。
	// capabilities はクライアントが利用可能な adapter 宣言（ClientCapabilities）。
	// service はフルサービス名（"k1s0.tier1.WorkflowSvc" 等）。
	// method は RPC メソッド名（"StreamWorkflowEvents" 等の bidi streaming method）。
	Negotiate(
		ctx context.Context,
		service string,
		method string,
		capabilities ClientCapabilities,
	) (BidiChannel, error)

	// BaseURL は接続先 Gateway の base URL を返す（デバッグ / ログ用）。
	BaseURL() string
}

// ---- NegotiationResult 型 ----

// NegotiationResult は Gateway との capability exchange の結果を宣言する型。
// アプリがデバッグ / ログ目的で確認できる情報を保持する。
type NegotiationResult struct {
	// ChosenTransport: Gateway が選択した transport 種別
	ChosenTransport TransportKind
	// GatewayVersion: Gateway が報告したサービスバージョン
	GatewayVersion string
	// NegotiationLatencyMs: negotiation フロー全体のレイテンシ（ミリ秒）
	NegotiationLatencyMs int64
	// ResumeToken: 初期セッションの resume_token（再接続時に使用する）
	ResumeToken string
}

// ---- in-memory BidiChannel 実装（テスト / ドライラン用） ----

// inMemoryBidiChannel は BidiChannel の in-memory stub 実装型。
// テスト / ドライラン用に送受信メッセージを channel 経由でシミュレートする。
type inMemoryBidiChannel struct {
	// chosenTransport: シミュレートする transport 種別
	chosenTransport TransportKind
	// caps: 起動時に送出したクライアント capability 宣言
	caps ClientCapabilities
	// sendCh: 送信メッセージを受け取るチャンネル（テストが読む）
	sendCh chan []byte
	// recvCh: 受信メッセージを注入するチャンネル（テストが書く）
	recvCh chan *BidiMessage
	// closed: Close が呼ばれたかどうかのフラグ
	closed bool
	// nextSeq: 送信メッセージのシーケンス番号カウンター
	nextSeq uint64
}

// NewInMemoryBidiChannel は inMemoryBidiChannel を生成するファクトリ関数（テスト用）。
// chosenTransport はシミュレートする transport 種別。
// bufferSize は send / recv チャンネルのバッファサイズ。
func NewInMemoryBidiChannel(chosenTransport TransportKind, bufferSize int) BidiChannel {
	// inMemoryBidiChannel を生成して返す
	return &inMemoryBidiChannel{
		chosenTransport: chosenTransport,
		caps:            DefaultClientCapabilities(),
		sendCh:          make(chan []byte, bufferSize),
		recvCh:          make(chan *BidiMessage, bufferSize),
	}
}

// Send はアプリ側からメッセージをチャンネルに送信する（テストが受け取る）。
func (c *inMemoryBidiChannel) Send(ctx context.Context, payload []byte) error {
	// Close 済みの場合はエラーを返す
	if c.closed {
		return errors.New("inMemoryBidiChannel: channel is closed")
	}
	// チャンネルにメッセージを送信する
	select {
	// ctx がキャンセルされた場合はエラーを返す
	case <-ctx.Done():
		return ctx.Err()
	// チャンネルにメッセージを送信する
	case c.sendCh <- payload:
		// シーケンス番号をインクリメントする
		c.nextSeq++
		return nil
	}
}

// Recv は受信チャンネルからメッセージを取得する（テストが注入したメッセージを返す）。
func (c *inMemoryBidiChannel) Recv(ctx context.Context) (*BidiMessage, error) {
	// Close 済みの場合はセッション終了として nil, nil を返す
	if c.closed {
		return nil, nil
	}
	// チャンネルからメッセージを受信する
	select {
	// ctx がキャンセルされた場合はエラーを返す
	case <-ctx.Done():
		return nil, ctx.Err()
	// チャンネルからメッセージを受信する
	case msg, ok := <-c.recvCh:
		// チャンネルが閉じられた場合はセッション終了として nil, nil を返す
		if !ok {
			return nil, nil
		}
		// メッセージを返す
		return msg, nil
	}
}

// Close はセッションをグレースフルに終了する（チャンネルを閉じる）。
func (c *inMemoryBidiChannel) Close(ctx context.Context) error {
	// closed フラグを立てる
	c.closed = true
	// 送信チャンネルを閉じる
	close(c.sendCh)
	// 完了を返す
	return nil
}

// ChosenTransport は現在アクティブな transport 種別を返す。
func (c *inMemoryBidiChannel) ChosenTransport() TransportKind {
	// chosenTransport フィールドを返す
	return c.chosenTransport
}

// Capabilities は起動時に送出したクライアント capability 宣言を返す。
func (c *inMemoryBidiChannel) Capabilities() ClientCapabilities {
	// caps フィールドを返す
	return c.caps
}

// InjectMessage はテスト用にメッセージを受信チャンネルに注入するヘルパーメソッド。
// 実装型の具体的メソッドとして提供する（BidiChannel interface 外）。
func (c *inMemoryBidiChannel) InjectMessage(msg *BidiMessage) error {
	// Close 済みの場合はエラーを返す
	if c.closed {
		return fmt.Errorf("inMemoryBidiChannel: channel is closed")
	}
	// 受信チャンネルにメッセージを注入する
	c.recvCh <- msg
	// 完了を返す
	return nil
}
