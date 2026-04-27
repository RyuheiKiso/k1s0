// 本ファイルは k1s0 .NET SDK の Binding 動詞統一 facade。
using Google.Protobuf;
using K1s0.Sdk.Generated.K1s0.Tier1.Binding.V1;

namespace K1s0.Sdk;

public sealed class BindingFacade
{
    private readonly K1s0Client _client;
    internal BindingFacade(K1s0Client client) { _client = client; }

    /// InvokeAsync: 出力バインディング呼出。
    public async Task<(byte[] Data, IReadOnlyDictionary<string, string> Metadata)> InvokeAsync(
        string name, string operation, byte[] data,
        IDictionary<string, string>? metadata = null, CancellationToken ct = default)
    {
        var req = new InvokeBindingRequest
        {
            Name = name,
            Operation = operation,
            Data = ByteString.CopyFrom(data),
            Context = _client.TenantContext(),
        };
        if (metadata != null) foreach (var kv in metadata) req.Metadata.Add(kv.Key, kv.Value);
        var resp = await _client.Raw.Binding.InvokeAsync(req, cancellationToken: ct);
        return (resp.Data.ToByteArray(), resp.Metadata);
    }
}
