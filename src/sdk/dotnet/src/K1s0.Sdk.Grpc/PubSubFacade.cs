// 本ファイルは k1s0 .NET SDK の PubSub 動詞統一 facade。
// `client.PubSub.PublishAsync(...)` 形式で PubSubService への呼出を提供する。

using Google.Protobuf;
using K1s0.Sdk.Generated.K1s0.Tier1.Pubsub.V1;

namespace K1s0.Sdk;

// Publish オプション。
public sealed class PublishOptions
{
    // 冪等性キー（24h 重複抑止）。
    public string IdempotencyKey { get; init; } = string.Empty;

    // メタデータ（partition_key / trace_id 等）。
    public IDictionary<string, string> Metadata { get; init; } = new Dictionary<string, string>();
}

// PubSubFacade は PubSubService の動詞統一 facade。
public sealed class PubSubFacade
{
    private readonly K1s0Client _client;

    internal PubSubFacade(K1s0Client client)
    {
        _client = client;
    }

    // PublishAsync は単発 Publish。Kafka offset を返す。
    public async Task<long> PublishAsync(
        string topic,
        byte[] data,
        string contentType,
        PublishOptions? opts = null,
        CancellationToken ct = default)
    {
        opts ??= new PublishOptions();
        var req = new PublishRequest
        {
            Topic = topic,
            Data = ByteString.CopyFrom(data),
            ContentType = contentType,
            IdempotencyKey = opts.IdempotencyKey,
            Context = _client.TenantContext(),
        };
        // metadata を追加する（proto map field）。
        foreach (var kv in opts.Metadata)
        {
            req.Metadata.Add(kv.Key, kv.Value);
        }
        // RPC 呼出。
        var resp = await _client.RawPubSub().PublishAsync(req, cancellationToken: ct);
        // offset を返却する。
        return resp.Offset;
    }
}
