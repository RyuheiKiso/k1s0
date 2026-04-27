// 本ファイルは k1s0 .NET SDK の ServiceInvoke 動詞統一 facade（unary + server streaming）。
using System.Runtime.CompilerServices;
using Google.Protobuf;
using K1s0.Sdk.Generated.K1s0.Tier1.Serviceinvoke.V1;

namespace K1s0.Sdk;

public sealed class InvokeFacade
{
    private readonly K1s0Client _client;
    internal InvokeFacade(K1s0Client client) { _client = client; }

    /// CallAsync: 任意サービスの任意メソッドを呼び出す（unary）。
    public async Task<(byte[] Data, string ContentType, int Status)> CallAsync(
        string appId, string method, byte[] data, string contentType, int timeoutMs = 5000, CancellationToken ct = default)
    {
        var resp = await _client.Raw.ServiceInvoke.InvokeAsync(new InvokeRequest
        {
            AppId = appId,
            Method = method,
            Data = ByteString.CopyFrom(data),
            ContentType = contentType,
            Context = _client.TenantContext(),
            TimeoutMs = timeoutMs,
        }, cancellationToken: ct);
        return (resp.Data.ToByteArray(), resp.ContentType, resp.Status);
    }

    /// StreamAsync: サーバストリーミング呼出。IAsyncEnumerable&lt;InvokeChunk&gt; を返す。
    /// 利用例:
    ///   await foreach (var chunk in client.Invoke.StreamAsync(appId, method, data, contentType))
    ///   {
    ///       Console.WriteLine(chunk.Data.Length);
    ///       if (chunk.Eof) break;
    ///   }
    public async IAsyncEnumerable<InvokeChunk> StreamAsync(
        string appId, string method, byte[] data, string contentType, int timeoutMs = 5000,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        using var call = _client.Raw.ServiceInvoke.InvokeStream(new InvokeRequest
        {
            AppId = appId,
            Method = method,
            Data = ByteString.CopyFrom(data),
            ContentType = contentType,
            Context = _client.TenantContext(),
            TimeoutMs = timeoutMs,
        }, cancellationToken: ct);
        // ResponseStream を await foreach で消費可能にする。
        while (await call.ResponseStream.MoveNext(ct))
        {
            yield return call.ResponseStream.Current;
        }
    }
}
