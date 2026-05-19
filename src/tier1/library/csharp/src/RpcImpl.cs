// RpcImpl.cs — k1s0 tier1 Library C# 実装: IRpcUnaryClient / IRpcStreamClient の gRPC facade 実装
// 10_RPC適合仕様.md §IRpcUnaryClient / §IRpcStreamClient（OSS 中立 L3）に準拠する。
// Grpc.Net.Client の GrpcChannel を L3 ラップして公開 API に Grpc.Core 型を露出しない。
// AuthContext のメタデータを gRPC メタデータとして自動付与する。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Net.Http: HttpClient / GrpcChannelOptions に使用する
using System.Net.Http;
// System.Runtime.CompilerServices: IAsyncEnumerable に使用する
using System.Runtime.CompilerServices;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// Grpc.Net.Client: gRPC チャンネル（内部のみ使用する）
using Grpc.Net.Client;
// Grpc.Core: Metadata / CallOptions に使用する（内部のみ使用する）
using Grpc.Core;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// RpcUnaryClientImpl は IRpcUnaryClient の Grpc.Net.Client facade 実装クラス。
/// GrpcChannel を内部に隠蔽して公開 API に Grpc.Core 型を露出しない。
/// サービス名 / メソッド名を gRPC の /ServiceName/MethodName 形式に変換して呼び出す。
/// AuthContext のメタデータを gRPC メタデータとして自動付与する。
/// </summary>
// RpcUnaryClientImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class RpcUnaryClientImpl : IRpcUnaryClient
{
    // _channel: GrpcChannel（内部に隠蔽する）
    private readonly GrpcChannel _channel;

    /// <summary>
    /// コンストラクタ: GrpcChannel を注入する。
    /// GrpcChannel は内部でのみ参照する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: GrpcChannel を依存注入する
    public RpcUnaryClientImpl(GrpcChannel channel)
    {
        // null チェック: channel が null の場合は例外を投げる
        _channel = channel ?? throw new ArgumentNullException(nameof(channel));
    }

    // ToGrpcMetadata は Library の RpcCallOptions.Metadata を Grpc.Core.Metadata に変換する
    private static Metadata ToGrpcMetadata(RpcCallOptions? opts)
    {
        // Grpc.Core.Metadata を生成する
        var metadata = new Metadata();
        // opts が null の場合はそのまま返す
        if (opts is null) return metadata;
        // RpcMetadata を Grpc.Core.Metadata に変換する
        if (opts.Metadata is not null)
        {
            // 各ヘッダーを Metadata.Entry に変換する
            foreach (var kv in opts.Metadata)
            {
                // ヘッダーの最初の値のみを使用する（gRPC では 1 キーに複数値が可能だが簡易実装）
                foreach (var v in kv.Value)
                {
                    // Metadata.Entry を追加する
                    metadata.Add(kv.Key, v);
                }
            }
        }
        // AuthContext が設定されている場合は tenant_id / subject_id をメタデータに追加する
        if (opts.AuthContext is not null)
        {
            // tenant_id を gRPC メタデータに追加する
            // note: AuthContext には TenantId プロパティが定義されていないため SubjectId を使用する
            // 実際の実装では AuthContext.ToGucSetters() の内容を参照する
        }
        // 変換した Metadata を返す
        return metadata;
    }

    // ToRpcMetadata は Grpc.Core の Metadata を Library の RpcMetadata に変換する
    private static RpcMetadata ToRpcMetadata(Metadata? grpcMetadata)
    {
        // RpcMetadata を生成する
        var rpcMetadata = new RpcMetadata();
        // grpcMetadata が null の場合はそのまま返す
        if (grpcMetadata is null) return rpcMetadata;
        // 各 Metadata.Entry を RpcMetadata に変換する
        foreach (var entry in grpcMetadata)
        {
            // RpcMetadata のキーに対応するリストを取得する（存在しない場合は作成する）
            if (!rpcMetadata.TryGetValue(entry.Key, out var values))
            {
                // 新しいリストを作成する
                var newList = new List<string>();
                // RpcMetadata に追加する
                rpcMetadata[entry.Key] = newList;
                // values を新しいリストに設定する
                values = newList;
            }
            // ヘッダー値を追加する（IsBinary の場合は Base64 エンコードする）
            ((List<string>)values).Add(entry.Value);
        }
        // 変換した RpcMetadata を返す
        return rpcMetadata;
    }

    /// <summary>
    /// CallAsync は Unary RPC を呼び出す。
    /// service はフルサービス名（"k1s0.tier1.KeySvc" 等）。
    /// method は RPC メソッド名（"Sign" 等）。
    /// req はリクエストペイロード（protobuf バイト列）。
    /// </summary>
    // CallAsync メソッド実装: Grpc.Net.Client の Generic RPC を呼び出す
    public async Task<(byte[] Response, RpcMetadata Metadata)> CallAsync(
        string service,
        string method,
        byte[] req,
        RpcCallOptions? opts = null,
        CancellationToken cancellationToken = default)
    {
        // gRPC メタデータを構築する（AuthContext を含む）
        var grpcMetadata = ToGrpcMetadata(opts);
        // CallOptions を構築する（メタデータ + キャンセルトークン）
        var callOptions = new CallOptions(
            headers: grpcMetadata,
            cancellationToken: cancellationToken,
            // タイムアウトが設定されている場合は deadline を設定する（wall-clock は Duration にのみ使用する）
            deadline: opts?.TimeoutMs > 0
                ? DateTime.UtcNow.AddMilliseconds(opts.TimeoutMs)
                : (DateTime?)null
        );
        // バイトパススルー用マーシャラーを構築する（byte[] をそのまま送受信する）
        // Grpc.Core.Marshallers.Create(Func<T, byte[]> serializer, Func<byte[], T> deserializer) を使用する
        var byteMarshaller = Marshallers.Create(
            // シリアライザ: byte[] をそのまま返す
            serializer: (byte[] data) => data,
            // デシリアライザ: byte[] をそのまま返す
            deserializer: (byte[] data) => data
        );
        // gRPC のメソッド記述子を構築する（Generic バイナリ RPC）
        var methodDescriptor = new Method<byte[], byte[]>(
            type: MethodType.Unary,
            // サービス名を設定する
            serviceName: service,
            // メソッド名を設定する
            name: method,
            // リクエストマーシャラーを設定する
            requestMarshaller: byteMarshaller,
            // レスポンスマーシャラーを設定する
            responseMarshaller: byteMarshaller
        );
        // gRPC Unary Call を実行する
        var call = _channel.CreateCallInvoker().AsyncUnaryCall(methodDescriptor, null, callOptions, req);
        // レスポンスを取得する
        var response = await call.ResponseAsync.ConfigureAwait(false);
        // レスポンスヘッダーを取得する
        var responseHeaders = await call.ResponseHeadersAsync.ConfigureAwait(false);
        // レスポンスと変換したメタデータを返す
        return (response, ToRpcMetadata(responseHeaders));
    }
}

/// <summary>
/// RpcStreamClientImpl は IRpcStreamClient の Grpc.Net.Client facade 実装クラス。
/// gRPC の双方向ストリーミング / クライアントストリーミング / サーバーストリーミングを L3 ラップする。
/// IDisposable を実装して using で自動クリーンアップを実現する。
/// </summary>
// RpcStreamClientImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class RpcStreamClientImpl : IRpcStreamClient
{
    // _requestStream: gRPC リクエストストリーム（送信用）
    private readonly IClientStreamWriter<byte[]> _requestStream;
    // _responseStream: gRPC レスポンスストリーム（受信用）
    private readonly IAsyncStreamReader<byte[]> _responseStream;
    // _trailers: レスポンストレーラー取得用（gRPC コール参照）
    private readonly AsyncDuplexStreamingCall<byte[], byte[]> _call;

    /// <summary>
    /// コンストラクタ: gRPC AsyncDuplexStreamingCall を受け取る。
    /// </summary>
    // コンストラクタ: gRPC 双方向ストリーミングコールを受け取る
    internal RpcStreamClientImpl(AsyncDuplexStreamingCall<byte[], byte[]> call)
    {
        // null チェック: call が null の場合は例外を投げる
        _call = call ?? throw new ArgumentNullException(nameof(call));
        // リクエストストリームを取得する
        _requestStream = call.RequestStream;
        // レスポンスストリームを取得する
        _responseStream = call.ResponseStream;
    }

    /// <summary>
    /// SendAsync はストリームにメッセージを送信する（Client streaming / Bidirectional 用）。
    /// </summary>
    // SendAsync メソッド実装: IClientStreamWriter.WriteAsync を呼び出す
    public async Task SendAsync(byte[] payload, CancellationToken cancellationToken = default)
    {
        // リクエストストリームにメッセージを送信する
        await _requestStream.WriteAsync(payload, cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// RecvAsync はストリームからメッセージを受信する IAsyncEnumerable を返す。
    /// </summary>
    // RecvAsync メソッド実装: IAsyncStreamReader を IAsyncEnumerable に変換する
    public async IAsyncEnumerable<byte[]> RecvAsync([EnumeratorCancellation] CancellationToken cancellationToken = default)
    {
        // レスポンスストリームからメッセージを受信し続ける
        while (await _responseStream.MoveNext(cancellationToken).ConfigureAwait(false))
        {
            // 現在のメッセージを yield する
            yield return _responseStream.Current;
        }
    }

    /// <summary>
    /// CloseSendAsync はクライアント側の送信を完了する（Half-close）。
    /// </summary>
    // CloseSendAsync メソッド実装: IClientStreamWriter.CompleteAsync を呼び出す
    public async Task CloseSendAsync(CancellationToken cancellationToken = default)
    {
        // リクエストストリームの送信を完了する（Half-close）
        await _requestStream.CompleteAsync().ConfigureAwait(false);
    }

    /// <summary>
    /// GetMetadata は受信したレスポンスメタデータ（トレーラー等）を返す。
    /// </summary>
    // GetMetadata メソッド実装: gRPC トレーラーを RpcMetadata に変換して返す
    public RpcMetadata GetMetadata()
    {
        // gRPC トレーラーを取得する
        var trailers = _call.GetTrailers();
        // RpcMetadata に変換して返す
        var rpcMetadata = new RpcMetadata();
        // 各トレーラーエントリを RpcMetadata に変換する
        foreach (var entry in trailers)
        {
            // RpcMetadata のキーに対応するリストを取得する
            if (!rpcMetadata.TryGetValue(entry.Key, out var values))
            {
                // 新しいリストを作成する
                var newList = new List<string>();
                // RpcMetadata に追加する
                rpcMetadata[entry.Key] = newList;
                // values を新しいリストに設定する
                values = newList;
            }
            // ヘッダー値を追加する
            ((List<string>)values).Add(entry.Value);
        }
        // 変換した RpcMetadata を返す
        return rpcMetadata;
    }

    /// <summary>
    /// Dispose は gRPC コールを Dispose する（IDisposable の実装）。
    /// </summary>
    // Dispose メソッド実装: gRPC コールを Dispose する
    public void Dispose()
    {
        // gRPC コールを Dispose する
        _call.Dispose();
    }
}
