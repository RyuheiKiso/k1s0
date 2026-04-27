// 本ファイルは k1s0 .NET SDK の PubSub 動詞統一 facade（publish + subscribe）。

using System.Runtime.CompilerServices;
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

    /// SubscribeAsync: トピックの購読。IAsyncEnumerable&lt;Event&gt; を返す。
    /// 利用例:
    ///   await foreach (var ev in client.PubSub.SubscribeAsync("orders", "consumer-A"))
    ///   {
    ///       Handle(ev);
    ///   }
    public async IAsyncEnumerable<Event> SubscribeAsync(
        string topic, string consumerGroup,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        using var call = _client.RawPubSub().Subscribe(new SubscribeRequest
        {
            Topic = topic,
            ConsumerGroup = consumerGroup,
            Context = _client.TenantContext(),
        }, cancellationToken: ct);
        while (await call.ResponseStream.MoveNext(ct))
        {
            yield return call.ResponseStream.Current;
        }
    }
}
