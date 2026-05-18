// IRpc.cs — k1s0 tier1 Library C# 実装: RPC / Gateway の L3 interface
// 10_RPC適合仕様.md §IRpcUnaryClient / §IGatewayHandler（OSS 中立 L3）に準拠する。
// gRPC / ConnectRPC / HTTP/2 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
// 公開シグネチャに OSS 型（Grpc.Core 等）を一切含まない。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;
// System.Runtime.CompilerServices: IAsyncEnumerable に使用する
using System.Runtime.CompilerServices;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// RpcMetadata は RPC 呼び出しのメタデータ（ヘッダー相当）を宣言する型。
/// OSS の Metadata / HttpHeaders を露出せず Library 独自語彙で表現する。
/// </summary>
// RpcMetadata 型エイリアス定義
public sealed class RpcMetadata : Dictionary<string, IReadOnlyList<string>>
{
    // RpcMetadata: string キーに string リスト値を持つ辞書（ヘッダー相当）
}

/// <summary>
/// RpcStatusCode は RPC 呼び出しのステータスコードを宣言する enum。
/// gRPC Status Code に準拠した Library 独自語彙とする。
/// </summary>
// RpcStatusCode 列挙型定義
public enum RpcStatusCode
{
    /// <summary>Ok: 成功（gRPC OK = 0）</summary>
    Ok = 0,
    /// <summary>Canceled: キャンセル（gRPC Canceled = 1）</summary>
    Canceled = 1,
    /// <summary>Unknown: 不明なエラー（gRPC Unknown = 2）</summary>
    Unknown = 2,
    /// <summary>InvalidArgument: 無効な引数（gRPC InvalidArgument = 3）</summary>
    InvalidArgument = 3,
    /// <summary>DeadlineExceeded: デッドライン超過（gRPC DeadlineExceeded = 4）</summary>
    DeadlineExceeded = 4,
    /// <summary>NotFound: リソース未発見（gRPC NotFound = 5）</summary>
    NotFound = 5,
    /// <summary>AlreadyExists: リソース重複（gRPC AlreadyExists = 6）</summary>
    AlreadyExists = 6,
    /// <summary>PermissionDenied: 権限エラー（gRPC PermissionDenied = 7）</summary>
    PermissionDenied = 7,
    /// <summary>ResourceExhausted: リソース枯渇（gRPC ResourceExhausted = 8）</summary>
    ResourceExhausted = 8,
    /// <summary>FailedPrecondition: 前提条件不満（gRPC FailedPrecondition = 9）</summary>
    FailedPrecondition = 9,
    /// <summary>Aborted: 中断（gRPC Aborted = 10）</summary>
    Aborted = 10,
    /// <summary>Internal: 内部エラー（gRPC Internal = 13）</summary>
    Internal = 13,
    /// <summary>Unavailable: サービス不可（gRPC Unavailable = 14）</summary>
    Unavailable = 14,
    /// <summary>Unauthenticated: 認証エラー（gRPC Unauthenticated = 16）</summary>
    Unauthenticated = 16,
}

/// <summary>
/// RpcException は RPC 呼び出しエラーを宣言する例外クラス。
/// OSS の RpcException / ConnectException を露出せず Library 独自語彙で表現する。
/// </summary>
// RpcException クラス定義
public sealed class RpcException : Exception
{
    /// <summary>Code: gRPC Status Code 準拠のステータスコード</summary>
    // Code プロパティ
    public RpcStatusCode Code { get; }

    /// <summary>コンストラクタ: code と message を受け取る</summary>
    // コンストラクタ: code と message を受け取る
    public RpcException(RpcStatusCode code, string message) : base(message)
    {
        // Code を設定する
        Code = code;
    }

    /// <summary>コンストラクタ: code / message / innerException を受け取る</summary>
    // コンストラクタ: code / message / innerException を受け取る
    public RpcException(RpcStatusCode code, string message, Exception innerException)
        : base(message, innerException)
    {
        // Code を設定する
        Code = code;
    }
}

/// <summary>
/// RpcCallOptions は RPC 呼び出しに渡すオプションを宣言する型。
/// </summary>
// RpcCallOptions クラス定義
public sealed class RpcCallOptions
{
    /// <summary>Metadata: 送信するリクエストメタデータ（ヘッダー相当）</summary>
    // Metadata プロパティ
    public RpcMetadata? Metadata { get; init; }

    /// <summary>AuthContext: RPC 呼び出しに付与する認証コンテキスト（tenant 分離に必須）</summary>
    // AuthContext プロパティ
    public AuthContext? AuthContext { get; init; }

    /// <summary>TimeoutMs: タイムアウトミリ秒（0 = CancellationToken に従う）</summary>
    // TimeoutMs プロパティ
    public long TimeoutMs { get; init; }
}

/// <summary>
/// IRpcUnaryClient は Unary RPC の L3 抽象 interface を宣言する。
/// OSS 型を引数・戻り値に一切含まない。
/// </summary>
// IRpcUnaryClient インターフェース定義
public interface IRpcUnaryClient
{
    /// <summary>
    /// CallAsync は Unary RPC を呼び出す。
    /// service はフルサービス名（"k1s0.tier1.KeySvc" 等）。
    /// method は RPC メソッド名（"Sign" 等）。
    /// req はリクエストペイロード（protobuf の JSON バイト列）。
    /// 戻り値はレスポンスペイロードとレスポンスメタデータ。
    /// </summary>
    // CallAsync メソッド: Unary RPC を呼び出す
    Task<(byte[] Response, RpcMetadata Metadata)> CallAsync(
        string service,
        string method,
        byte[] req,
        RpcCallOptions? opts = null,
        CancellationToken cancellationToken = default);
}

/// <summary>
/// IRpcStreamClient は Streaming RPC の L3 抽象 interface を宣言する。
/// </summary>
// IRpcStreamClient インターフェース定義
public interface IRpcStreamClient : IDisposable
{
    /// <summary>
    /// SendAsync はストリームにメッセージを送信する（Client streaming / Bidirectional 用）。
    /// </summary>
    // SendAsync メソッド: ストリームにメッセージを送信する
    Task SendAsync(byte[] payload, CancellationToken cancellationToken = default);

    /// <summary>
    /// RecvAsync はストリームからメッセージを受信する IAsyncEnumerable を返す。
    /// </summary>
    // RecvAsync メソッド: ストリームからメッセージを受信する
    IAsyncEnumerable<byte[]> RecvAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// CloseSendAsync はクライアント側の送信を完了する（Half-close）。
    /// </summary>
    // CloseSendAsync メソッド: 送信を完了する
    Task CloseSendAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// GetMetadata は受信したレスポンスメタデータ（トレーラー等）を返す。
    /// </summary>
    // GetMetadata メソッド: レスポンスメタデータを返す
    RpcMetadata GetMetadata();
}

/// <summary>
/// GatewayRequest は Gateway 経由の HTTP/gRPC リクエストを宣言する型。
/// OSS の HttpRequest を露出せず Library 独自語彙で表現する。
/// </summary>
// GatewayRequest クラス定義
public sealed class GatewayRequest
{
    /// <summary>Method: HTTP メソッド（"GET" / "POST" 等）</summary>
    // Method プロパティ（必須）
    public required string Method { get; init; }

    /// <summary>Path: リクエストパス（"/v1/keys/sign" 等）</summary>
    // Path プロパティ（必須）
    public required string Path { get; init; }

    /// <summary>Headers: リクエストヘッダー</summary>
    // Headers プロパティ（必須）
    public required RpcMetadata Headers { get; init; }

    /// <summary>Body: リクエストボディバイト列</summary>
    // Body プロパティ（必須）
    public required byte[] Body { get; init; }

    /// <summary>AuthContext: Gateway が検証済み認証コンテキスト（tenant 分離必須）</summary>
    // AuthContext プロパティ（必須）
    public required AuthContext AuthContext { get; init; }
}

/// <summary>
/// GatewayResponse は Gateway から返す HTTP レスポンスを宣言する型。
/// </summary>
// GatewayResponse クラス定義
public sealed class GatewayResponse
{
    /// <summary>StatusCode: HTTP ステータスコード</summary>
    // StatusCode プロパティ（必須）
    public required int StatusCode { get; init; }

    /// <summary>Headers: レスポンスヘッダー</summary>
    // Headers プロパティ（必須）
    public required RpcMetadata Headers { get; init; }

    /// <summary>Body: レスポンスボディバイト列</summary>
    // Body プロパティ（必須）
    public required byte[] Body { get; init; }
}

/// <summary>
/// IGatewayHandler は Gateway のリクエストハンドラー interface を宣言する。
/// </summary>
// IGatewayHandler インターフェース定義
public interface IGatewayHandler
{
    /// <summary>
    /// HandleAsync は GatewayRequest を受け取って GatewayResponse を返す。
    /// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
    /// </summary>
    // HandleAsync メソッド: リクエストを処理する
    Task<GatewayResponse> HandleAsync(GatewayRequest req, CancellationToken cancellationToken = default);
}
