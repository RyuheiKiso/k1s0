// TransportNegotiation.cs — k1s0 tier1 Library C#: Transport Negotiation Runtime
// docs/03_概要設計/02_tier1設計方針/02_Library.md §Companion 役割 B に準拠する。
// tier1 Server 系 Transport Adapter Layer と対をなすクライアント実装を提供する。
// 8 adapter（sse_paired / long_poll / webhook / websocket / web_transport / messaging_bridge
//            / grpc_web / connect_rpc）の chosen_transport capability negotiation を担う。
// Rust frontend/transport_negotiation.rs / Go frontend/transport_negotiation.go /
// TypeScript frontend/transport_negotiation.ts と 4 言語等価強度を保つ。
// OSS の transport 型（System.Net.Http.HttpClient / System.Net.WebSockets 等）を
// 公開 API に一切露出しない。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyList に使用する
using System.Collections.Generic;
// System.Collections.Concurrent: ConcurrentQueue に使用する
using System.Collections.Concurrent;
// System.Runtime.CompilerServices: IAsyncEnumerable に使用する
using System.Runtime.CompilerServices;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Channels: Channel<T> に使用する
using System.Threading.Channels;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;

// k1s0 tier1 Frontend 名前空間（backend 専用の依存が混入しないよう分離する）
namespace K1s0.Tier1.Frontend;

// ---- Transport adapter 種別定義 ----

/// <summary>
/// TransportKind は tier1 Server が選択可能な 8 transport adapter 種別を宣言する enum。
/// Gateway の chosen_transport フィールドと 1:1 対応する Library 独自語彙とする。
/// Rust TransportKind / Go TransportKind / TypeScript TransportKind と 4 言語等価強度を保つ。
/// </summary>
// TransportKind 列挙型定義
public enum TransportKind
{
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
    // MessagingBridge: Kafka / AMQP の REST Proxy 越しに bidi メッセージを搬送する
    MessagingBridge,
    // GrpcWeb: gRPC-Web プロトコル（HTTP/1.1 対応環境で gRPC を使用する場合）
    GrpcWeb,
    // ConnectRpc: ConnectRPC プロトコル（gRPC / gRPC-Web との相互運用性が高い）
    ConnectRpc,
}

// ---- クライアント Capability 宣言 ----

/// <summary>
/// ClientCapabilities は Companion が起動時に Gateway に送出する capability 宣言を定義するクラス。
/// 利用可能な adapter 一覧 / TLS バージョン / inbound 可否 / max message size 等を宣言する。
/// Open RPC の client_capabilities 形式と互換性を保つ。
/// Rust ClientCapabilities / Go ClientCapabilities / TypeScript ClientCapabilities と 4 言語等価強度を保つ。
/// </summary>
// ClientCapabilities クラス定義（record で不変性を保証する）
public sealed record ClientCapabilities(
    // AvailableTransports: クライアントが利用可能な transport adapter 一覧
    // Gateway はこの一覧から chosen_transport を選択する
    IReadOnlyList<TransportKind> AvailableTransports,
    // TlsMinVersion: クライアントがサポートする TLS 最低バージョン（例: "TLSv1.2" / "TLSv1.3"）
    string TlsMinVersion,
    // InboundCapable: Webhook adapter を受信できるかどうか（HTTP サーバーとして動作可能な場合 true）
    bool InboundCapable,
    // MaxMessageSizeBytes: 1 メッセージの最大サイズ（バイト）
    // 0 = 制限なし（実装側の OS / stack の制限に従う）
    ulong MaxMessageSizeBytes,
    // ResumeTokenSupport: resume_token による再接続をサポートするかどうか
    // SsePaired / WebSocket / WebTransport adapter で有効化する
    bool ResumeTokenSupport
)
{
    /// <summary>
    /// CreateDefault は frontend 向け安全なデフォルト ClientCapabilities を返すファクトリメソッド。
    /// SSE + LongPoll + WebSocket の 3 adapter をデフォルトで利用可能とする。
    /// </summary>
    // CreateDefault ファクトリメソッド: デフォルト ClientCapabilities を返す
    public static ClientCapabilities CreateDefault()
    {
        // frontend 向け安全なデフォルト設定を返す
        return new ClientCapabilities(
            // SSE + LongPoll + WebSocket の 3 adapter をデフォルトで利用可能とする
            AvailableTransports: new[] { TransportKind.SsePaired, TransportKind.LongPoll, TransportKind.WebSocket },
            // TLS 1.2 をデフォルト最低バージョンとする（TLS 1.3 推奨だが互換性のため 1.2 を下限とする）
            TlsMinVersion: "TLSv1.2",
            // デフォルトでは inbound を受け入れない（Webhook opt-in が必要）
            InboundCapable: false,
            // デフォルト最大メッセージサイズ: 4MB（大半の業務 RPC に十分な値）
            MaxMessageSizeBytes: 4 * 1024 * 1024,
            // デフォルトで resume_token をサポートする（SsePaired の id: フィールドを使用する）
            ResumeTokenSupport: true
        );
    }
}

// ---- 双方向チャンネル抽象 ----

/// <summary>
/// BidiMessage はアプリ側が送受信する双方向メッセージを宣言するレコード型。
/// transport 種別を意識しない統一型（プロトコルバッファのバイト列を運ぶ）。
/// Rust BidiMessage / Go BidiMessage / TypeScript BidiMessage と 4 言語等価強度を保つ。
/// </summary>
// BidiMessage レコード型定義
public sealed record BidiMessage(
    // Payload: メッセージボディ（protobuf バイト列）
    byte[] Payload,
    // SequenceId: メッセージ順序番号（resume_token 透過化に使用する）
    // 送信側が単調増加で付与し、受信側は順序検証に使用する
    ulong SequenceId,
    // ResumeToken: セッション再接続トークン（transport 切替後の継続に使用する）
    // null の場合はトークンなし（SsePaired が初回接続など）
    string? ResumeToken = null
);

// ---- IBidiChannel interface ----

/// <summary>
/// IBidiChannel は双方向 RPC セッションのアプリ向け抽象 interface を宣言する。
/// アプリ側は transport 種別を一切意識せず SendAsync / RecvAsync の API のみを使用する。
/// 再接続は IBidiChannel 実装内部で透過的に処理される（アプリに漏れない）。
/// Rust BidiChannel / Go BidiChannel / TypeScript BidiChannel と 4 言語等価強度を保つ。
/// </summary>
// IBidiChannel インターフェース定義
public interface IBidiChannel : IAsyncDisposable
{
    /// <summary>
    /// SendAsync はアプリ側からメッセージを送信する。
    /// transport が切り替わっても呼び出し元は継続できる（透過再接続）。
    /// </summary>
    // SendAsync メソッド: メッセージを送信する
    Task SendAsync(byte[] payload, CancellationToken cancellationToken = default);

    /// <summary>
    /// RecvAsync はストリームからメッセージを受信する IAsyncEnumerable を返す。
    /// セッション終了まで受信し続ける（transport 切替による中断は内部でハンドルする）。
    /// </summary>
    // RecvAsync メソッド: 受信メッセージを IAsyncEnumerable で返す
    IAsyncEnumerable<BidiMessage> RecvAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// CloseAsync はセッションをグレースフルに終了する（送信未完了メッセージをフラッシュする）。
    /// </summary>
    // CloseAsync メソッド: セッションを終了する
    Task CloseAsync(CancellationToken cancellationToken = default);

    /// <summary>ChosenTransport は現在アクティブな transport 種別を返す（デバッグ / ログ用）。</summary>
    // ChosenTransport プロパティ: アクティブな transport 種別
    TransportKind ChosenTransport { get; }

    /// <summary>Capabilities は起動時に送出したクライアント capability 宣言を返す（デバッグ用）。</summary>
    // Capabilities プロパティ: クライアント capability 宣言
    ClientCapabilities Capabilities { get; }
}

// ---- ITransportNegotiationClient interface ----

/// <summary>
/// ITransportNegotiationClient は transport negotiation を行って IBidiChannel を開設する facade interface。
/// Gateway との negotiation フローを抽象化し、アプリ側は transport 選択結果を受け取るだけでよい。
/// Rust TransportNegotiationClient / Go TransportNegotiationClient /
/// TypeScript TransportNegotiationClient と 4 言語等価強度を保つ。
/// </summary>
// ITransportNegotiationClient インターフェース定義
public interface ITransportNegotiationClient
{
    /// <summary>
    /// NegotiateAsync は Gateway との capability exchange を行い、最適な IBidiChannel を返す。
    /// capabilities はクライアントが利用可能な adapter 宣言（ClientCapabilities）。
    /// service はフルサービス名（"k1s0.tier1.WorkflowSvc" 等）。
    /// method は RPC メソッド名（"StreamWorkflowEvents" 等の bidi streaming method）。
    /// </summary>
    // NegotiateAsync メソッド: transport negotiation を行って IBidiChannel を返す
    Task<IBidiChannel> NegotiateAsync(
        // service: フルサービス名（"k1s0.tier1.WorkflowSvc" 等）
        string service,
        // method: RPC メソッド名（"StreamWorkflowEvents" 等の bidi streaming method）
        string method,
        // capabilities: クライアントが利用可能な adapter 宣言
        ClientCapabilities capabilities,
        // cancellationToken: キャンセルトークン
        CancellationToken cancellationToken = default
    );

    /// <summary>BaseUrl は接続先 Gateway の base URL を返す（デバッグ / ログ用）。</summary>
    // BaseUrl プロパティ: Gateway の base URL
    string BaseUrl { get; }
}

// ---- NegotiationResult 型 ----

/// <summary>
/// NegotiationResult は Gateway との capability exchange の結果を宣言するレコード型。
/// アプリがデバッグ / ログ目的で確認できる情報を保持する。
/// </summary>
// NegotiationResult レコード型定義
public sealed record NegotiationResult(
    // ChosenTransport: Gateway が選択した transport 種別
    TransportKind ChosenTransport,
    // GatewayVersion: Gateway が報告したサービスバージョン
    string GatewayVersion,
    // NegotiationLatencyMs: negotiation フロー全体のレイテンシ（ミリ秒）
    long NegotiationLatencyMs,
    // ResumeToken: 初期セッションの resume_token（再接続時に使用する）
    string? ResumeToken = null
);

// ---- InMemoryBidiChannel 実装（テスト / ドライラン用） ----

/// <summary>
/// InMemoryBidiChannel は IBidiChannel の in-memory stub 実装クラス。
/// テスト / ドライラン用に送受信メッセージを Channel&lt;T&gt; 経由でシミュレートする。
/// Rust inMemoryBidiChannel / Go inMemoryBidiChannel / TypeScript InMemoryBidiChannel と
/// 4 言語等価強度を保つ。
/// </summary>
// InMemoryBidiChannel クラス定義（テスト用の公開クラス）
public sealed class InMemoryBidiChannel : IBidiChannel
{
    // _chosenTransport: シミュレートする transport 種別
    private readonly TransportKind _chosenTransport;
    // _caps: 起動時に送出したクライアント capability 宣言
    private readonly ClientCapabilities _caps;
    // _sendQueue: 送信メッセージキュー（テストが読む）
    private readonly ConcurrentQueue<byte[]> _sendQueue = new();
    // _recvChannel: 受信メッセージチャンネル（テストが注入する）
    private readonly Channel<BidiMessage> _recvChannel;
    // _closed: Close が呼ばれたかどうかのフラグ
    private volatile bool _closed;
    // _nextSeq: 送信メッセージのシーケンス番号カウンター
    private ulong _nextSeq;

    /// <summary>
    /// コンストラクタ: chosenTransport と caps を受け取る。
    /// </summary>
    // コンストラクタ: chosenTransport と caps を受け取る
    public InMemoryBidiChannel(TransportKind chosenTransport, ClientCapabilities? caps = null)
    {
        // transport 種別を設定する
        _chosenTransport = chosenTransport;
        // capabilities を設定する（null の場合はデフォルト値を使用する）
        _caps = caps ?? ClientCapabilities.CreateDefault();
        // 受信チャンネルを生成する（UnboundedChannel で無制限バッファを使用する）
        _recvChannel = Channel.CreateUnbounded<BidiMessage>();
    }

    /// <summary>ChosenTransport は現在アクティブな transport 種別を返す。</summary>
    // ChosenTransport プロパティ実装
    public TransportKind ChosenTransport => _chosenTransport;

    /// <summary>Capabilities は起動時に送出したクライアント capability 宣言を返す。</summary>
    // Capabilities プロパティ実装
    public ClientCapabilities Capabilities => _caps;

    /// <summary>
    /// SendAsync はアプリ側からメッセージをキューに送信する（テストが受け取る）。
    /// </summary>
    // SendAsync メソッド実装: キューにメッセージを追加する
    public Task SendAsync(byte[] payload, CancellationToken cancellationToken = default)
    {
        // close 済みの場合はエラーを投げる
        if (_closed) throw new InvalidOperationException("InMemoryBidiChannel: channel is closed");
        // キューにメッセージを追加する
        _sendQueue.Enqueue(payload);
        // シーケンス番号をインクリメントする
        _nextSeq++;
        // 完了を返す
        return Task.CompletedTask;
    }

    /// <summary>
    /// RecvAsync は受信チャンネルからメッセージを受信する IAsyncEnumerable を返す。
    /// テストが InjectMessage で注入したメッセージを返す。
    /// </summary>
    // RecvAsync メソッド実装: 受信チャンネルからメッセージを yield する
    public async IAsyncEnumerable<BidiMessage> RecvAsync([EnumeratorCancellation] CancellationToken cancellationToken = default)
    {
        // チャンネルリーダーからメッセージを読み続ける
        await foreach (var msg in _recvChannel.Reader.ReadAllAsync(cancellationToken).ConfigureAwait(false))
        {
            // メッセージを yield する
            yield return msg;
        }
    }

    /// <summary>CloseAsync はセッションをグレースフルに終了する。</summary>
    // CloseAsync メソッド実装: closed フラグを立てて受信チャンネルを完了する
    public Task CloseAsync(CancellationToken cancellationToken = default)
    {
        // closed フラグを立てる
        _closed = true;
        // 受信チャンネルを完了する（RecvAsync のイテレーションを停止させる）
        _recvChannel.Writer.TryComplete();
        // 完了を返す
        return Task.CompletedTask;
    }

    /// <summary>DisposeAsync は CloseAsync を呼び出して IAsyncDisposable を実装する。</summary>
    // DisposeAsync メソッド実装: IAsyncDisposable の実装
    public async ValueTask DisposeAsync()
    {
        // CloseAsync を呼び出してリソースを解放する
        await CloseAsync().ConfigureAwait(false);
    }

    /// <summary>
    /// InjectMessage はテスト用にメッセージを受信チャンネルに注入するヘルパーメソッド。
    /// IBidiChannel interface 外のメソッド（テスト用）。
    /// </summary>
    // InjectMessage メソッド: 受信チャンネルにメッセージを注入する（テスト用）
    public void InjectMessage(BidiMessage msg)
    {
        // close 済みの場合はエラーを投げる
        if (_closed) throw new InvalidOperationException("InMemoryBidiChannel: channel is closed");
        // 受信チャンネルにメッセージを書き込む
        _recvChannel.Writer.TryWrite(msg);
    }

    /// <summary>SentMessages は送信済みメッセージキューを返す（テスト検証用）。</summary>
    // SentMessages プロパティ: 送信済みメッセージキュー（テスト用）
    public IReadOnlyCollection<byte[]> SentMessages => _sendQueue;
}
