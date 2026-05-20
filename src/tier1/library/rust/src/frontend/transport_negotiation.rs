// transport_negotiation.rs — k1s0 tier1 Library frontend: Transport Negotiation Runtime
// docs/03_概要設計/02_tier1設計方針/02_Library.md §Companion 役割 B に準拠する。
// tier1 Server 系 Transport Adapter Layer と対をなすクライアント実装を提供する。
// 8 adapter（sse_paired / long_poll / webhook / websocket / web_transport / messaging_bridge
//            / grpc_web / connect_rpc）の chosen_transport capability negotiation を担う。
// OSS の transport 型（reqwest / tungstenite / h3 等）を公開 API に一切露出しない。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: Serialize / Deserialize（capability 宣言の JSON シリアライズに使用する）
use serde::{Deserialize, Serialize};

// ---- Transport adapter の種別定義 ----

// TransportKind は tier1 Server が選択可能な 8 transport adapter 種別を宣言する型。
// Gateway の chosen_transport フィールドと 1:1 対応する Library 独自語彙とする。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportKind {
    // SsePaired: 既定。SSE フレーミング + 送信用 unary POST の組合せ
    // resume_token は SSE id: フィールド / Last-Event-ID ヘッダーで透過再接続する
    SsePaired,
    // LongPoll: cursor 付き short poll の組合せ（SSE 非対応環境向け）
    LongPoll,
    // Webhook: レガシー側が HTTP サーバーとして受信する opt-in adapter
    Webhook,
    // WebSocket: WebSocket ベース（HTTP Upgrade 対応環境向け）
    WebSocket,
    // WebTransport: QUIC / WebTransport クライアント向け（v1 では opt-in adapter 扱い）
    WebTransport,
    // MessagingBridge: Kafka / AMQP の REST Proxy 越しに bidi メッセージを搬送する形態
    MessagingBridge,
    // GrpcWeb: gRPC-Web プロトコル（HTTP/1.1 対応環境で gRPC を使用する場合）
    GrpcWeb,
    // ConnectRpc: ConnectRPC プロトコル（gRPC / gRPC-Web との相互運用性が高い）
    ConnectRpc,
}

// ---- クライアント Capability 宣言 ----

// ClientCapabilities は Companion が起動時に Gateway に送出する capability 宣言を定義する型。
// 利用可能な adapter 一覧 / TLS バージョン / inbound 可否 / max message size 等を宣言する。
// Open RPC の client_capabilities 形式と互換性を保つ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    // available_transports: クライアントが利用可能な transport adapter 一覧
    // Gateway はこの一覧から chosen_transport を選択する
    pub available_transports: Vec<TransportKind>,
    // tls_min_version: クライアントがサポートする TLS 最低バージョン（例: "TLSv1.2" / "TLSv1.3"）
    pub tls_min_version: String,
    // inbound_capable: Webhook adapter を受信できるかどうか（HTTP サーバーとして動作可能な場合 true）
    pub inbound_capable: bool,
    // max_message_size_bytes: 1 メッセージの最大サイズ（バイト）
    // 0 = 制限なし（実装側の OS / stack の制限に従う）
    pub max_message_size_bytes: u64,
    // resume_token_support: resume_token による再接続をサポートするかどうか
    // SsePaired / WebSocket / WebTransport adapter で有効化する
    pub resume_token_support: bool,
}

// ClientCapabilities のデフォルト値
impl Default for ClientCapabilities {
    fn default() -> Self {
        // frontend 向け安全なデフォルト設定
        Self {
            // SSE + LongPoll + WebSocket の 3 adapter をデフォルトで利用可能とする
            available_transports: vec![
                TransportKind::SsePaired,
                TransportKind::LongPoll,
                TransportKind::WebSocket,
            ],
            // TLS 1.2 をデフォルト最低バージョンとする（TLS 1.3 推奨だが互換性のため 1.2 を下限とする）
            tls_min_version: "TLSv1.2".to_string(),
            // デフォルトでは inbound を受け入れない（Webhook opt-in が必要）
            inbound_capable: false,
            // デフォルト最大メッセージサイズ: 4MB（大半の業務 RPC に十分な値）
            max_message_size_bytes: 4 * 1024 * 1024,
            // デフォルトで resume_token をサポートする（SsePaired の id: フィールドを使用する）
            resume_token_support: true,
        }
    }
}

// ---- 双方向チャンネル抽象 ----

// BidiMessage はアプリ側が送受信する双方向メッセージを宣言する型。
// transport 種別を意識しない統一型（プロトコルバッファのバイト列を運ぶ）。
#[derive(Debug, Clone)]
pub struct BidiMessage {
    // payload: メッセージボディ（protobuf バイト列）
    pub payload: Vec<u8>,
    // sequence_id: メッセージ順序番号（resume_token 透過化に使用する）
    // 送信側が単調増加で付与し、受信側は順序検証に使用する
    pub sequence_id: u64,
    // resume_token: セッション再接続トークン（transport 切替後の継続に使用する）
    // SsePaired の場合は SSE id: フィールド、WebSocket の場合は拡張ヘッダーで搬送する
    pub resume_token: Option<String>,
}

// ---- BidiChannel trait ----

// BidiChannel は双方向 RPC セッションのアプリ向け抽象 trait を宣言する。
// アプリ側は transport 種別を一切意識せず onMessage / send の API のみを使用する。
// 再接続は BidiChannel 実装内部で透過的に処理される（アプリに漏れない）。
#[async_trait]
pub trait BidiChannel: Send + Sync {
    // send はアプリ側からメッセージを送信する。
    // transport が切り替わっても呼び出し元は継続できる（透過再接続）。
    async fn send(&self, payload: Vec<u8>) -> Result<()>;

    // recv は受信メッセージを取得する非同期イテレーターを返す。
    // セッション終了まで受信し続ける（transport 切替による中断は内部でハンドルする）。
    async fn recv(&self) -> Result<Option<BidiMessage>>;

    // close はセッションをグレースフルに終了する（送信未完了メッセージをフラッシュする）。
    async fn close(&self) -> Result<()>;

    // chosen_transport は現在アクティブな transport 種別を返す（デバッグ / ログ用）。
    fn chosen_transport(&self) -> TransportKind;

    // capabilities は起動時に送出したクライアント capability 宣言を返す（デバッグ用）。
    fn capabilities(&self) -> &ClientCapabilities;
}

// ---- TransportNegotiationClient trait ----

// TransportNegotiationClient は transport negotiation を行って BidiChannel を開設する facade trait。
// Gateway との negotiation フローを抽象化し、アプリ側は transport 選択結果を受け取るだけでよい。
#[async_trait]
pub trait TransportNegotiationClient: Send + Sync {
    // negotiate は Gateway との capability exchange を行い、最適な BidiChannel を返す。
    // capabilities はクライアントが利用可能な adapter 宣言（ClientCapabilities）。
    // service はフルサービス名（"k1s0.tier1.WorkflowSvc" 等）。
    // method は RPC メソッド名（"StreamWorkflowEvents" 等の bidi streaming method）。
    async fn negotiate(
        &self,
        service: &str,
        method: &str,
        capabilities: &ClientCapabilities,
    ) -> Result<Box<dyn BidiChannel>>;

    // base_url は接続先 Gateway の base URL を返す（デバッグ / ログ用）。
    fn base_url(&self) -> &str;
}

// ---- SsePairedAdapter ----

// SsePairedChannelOptions は SsePaired adapter の設定を宣言する型。
// SSE + unary POST の組合せ接続に必要なパラメータを保持する。
#[derive(Debug, Clone)]
pub struct SsePairedChannelOptions {
    // sse_endpoint: SSE ストリームの受信エンドポイント URL
    pub sse_endpoint: String,
    // post_endpoint: メッセージ送信用の unary POST エンドポイント URL
    pub post_endpoint: String,
    // reconnect_delay_ms: 切断後の再接続待機時間（ミリ秒）
    pub reconnect_delay_ms: u64,
    // max_reconnect_attempts: 最大再接続試行回数（0 = 無制限）
    pub max_reconnect_attempts: u32,
}

// SsePairedChannelOptions のデフォルト値
impl Default for SsePairedChannelOptions {
    fn default() -> Self {
        // SSE adapter の安全なデフォルト設定
        Self {
            // sse_endpoint と post_endpoint はアプリが設定する（空文字列はエラー）
            sse_endpoint: String::new(),
            post_endpoint: String::new(),
            // 再接続待機時間: 1000ms（指数バックオフの初期値）
            reconnect_delay_ms: 1000,
            // 最大再接続試行回数: 10 回（0 = 無制限）
            max_reconnect_attempts: 10,
        }
    }
}

// ---- LongPollAdapter ----

// LongPollChannelOptions は LongPoll adapter の設定を宣言する型。
// cursor 付き short poll の接続パラメータを保持する。
#[derive(Debug, Clone)]
pub struct LongPollChannelOptions {
    // poll_endpoint: ポーリングエンドポイント URL（cursor パラメータを付与して呼び出す）
    pub poll_endpoint: String,
    // send_endpoint: メッセージ送信エンドポイント URL
    pub send_endpoint: String,
    // poll_interval_ms: ポーリング間隔（ミリ秒）
    pub poll_interval_ms: u64,
    // poll_timeout_ms: 1 回のポーリングのタイムアウト（ミリ秒）
    pub poll_timeout_ms: u64,
}

// LongPollChannelOptions のデフォルト値
impl Default for LongPollChannelOptions {
    fn default() -> Self {
        // LongPoll adapter の安全なデフォルト設定
        Self {
            // poll_endpoint と send_endpoint はアプリが設定する
            poll_endpoint: String::new(),
            send_endpoint: String::new(),
            // ポーリング間隔: 500ms（レイテンシと負荷のバランス点）
            poll_interval_ms: 500,
            // ポーリングタイムアウト: 30000ms（Long Poll の典型的タイムアウト）
            poll_timeout_ms: 30_000,
        }
    }
}

// ---- capability negotiation result ----

// NegotiationResult は Gateway との capability exchange の結果を宣言する型。
// アプリがデバッグ / ログ目的で確認できる情報を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationResult {
    // chosen_transport: Gateway が選択した transport 種別
    pub chosen_transport: TransportKind,
    // gateway_version: Gateway が報告したサービスバージョン
    pub gateway_version: String,
    // negotiation_latency_ms: negotiation フロー全体のレイテンシ（ミリ秒）
    pub negotiation_latency_ms: u64,
    // resume_token: 初期セッションの resume_token（再接続時に使用する）
    pub resume_token: Option<String>,
}
