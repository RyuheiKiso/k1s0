// 本ファイルは k1s0 .NET SDK の State 動詞統一 facade。
// `client.State.SaveAsync(...)` 形式で StateService への呼出を提供する。

using Google.Protobuf;
using K1s0.Sdk.Generated.K1s0.Tier1.State.V1;

namespace K1s0.Sdk;

// Save / Delete のオプション。
public sealed class SaveOptions
{
    // 期待 ETag（楽観的排他、空文字は無条件）。
    public string ExpectedEtag { get; init; } = string.Empty;

    // TTL 秒（0 / 省略は永続）。
    public int TtlSec { get; init; }

    // 冪等性キー（共通規約 §「冪等性と再試行」: 24h 重複抑止、空文字 / 省略で dedup 無効）。
    public string IdempotencyKey { get; init; } = string.Empty;
}

// StateFacade は StateService の動詞統一 facade。
public sealed class StateFacade
{
    // 親 Client への参照。
    private readonly K1s0Client _client;

    internal StateFacade(K1s0Client client)
    {
        _client = client;
    }

    // GetAsync はキー単位の取得。未存在時は null を返す。
    public async Task<(byte[] Data, string Etag)?> GetAsync(string store, string key, CancellationToken ct = default)
    {
        // proto Request を構築する。
        var req = new GetRequest
        {
            Store = store,
            Key = key,
            Context = _client.TenantContext(),
        };
        // 生成 stub 経由で RPC 呼び出し。
        var resp = await _client.RawState().GetAsync(req, cancellationToken: ct);
        // 未存在時は null。
        if (resp.NotFound)
        {
            return null;
        }
        // 存在時は (Data, Etag) を返却する。
        return (resp.Data.ToByteArray(), resp.Etag);
    }

    // SaveAsync はキー単位の保存。新 ETag を返す。
    public async Task<string> SaveAsync(
        string store,
        string key,
        byte[] data,
        SaveOptions? opts = null,
        CancellationToken ct = default)
    {
        // option を既定値で補完する。
        opts ??= new SaveOptions();
        // proto Request を構築する。
        var req = new SetRequest
        {
            Store = store,
            Key = key,
            Data = ByteString.CopyFrom(data),
            ExpectedEtag = opts.ExpectedEtag,
            TtlSec = opts.TtlSec,
            IdempotencyKey = opts.IdempotencyKey,
            Context = _client.TenantContext(),
        };
        // RPC 呼出。
        var resp = await _client.RawState().SetAsync(req, cancellationToken: ct);
        // 新 ETag を返却する。
        return resp.NewEtag;
    }

    // DeleteAsync はキー単位の削除。expected_etag が空なら無条件。
    public async Task<bool> DeleteAsync(
        string store,
        string key,
        string expectedEtag = "",
        CancellationToken ct = default)
    {
        // proto Request を構築する。
        var req = new DeleteRequest
        {
            Store = store,
            Key = key,
            ExpectedEtag = expectedEtag,
            Context = _client.TenantContext(),
        };
        // RPC 呼出。
        var resp = await _client.RawState().DeleteAsync(req, cancellationToken: ct);
        // deleted フラグを返却する。
        return resp.Deleted;
    }
}
