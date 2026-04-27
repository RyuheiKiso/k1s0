// 本ファイルは k1s0 .NET SDK の ServiceInvoke 動詞統一 facade。
// InvokeStream は本リリース時点 では Raw 経由（client.Raw.ServiceInvoke から直接アクセス）。
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
}
