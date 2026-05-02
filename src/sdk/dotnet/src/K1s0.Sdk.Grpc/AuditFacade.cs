// 本ファイルは k1s0 .NET SDK の Audit 動詞統一 facade。
using System.Runtime.CompilerServices;
using Google.Protobuf.WellKnownTypes;
using K1s0.Sdk.Generated.K1s0.Tier1.Audit.V1;

namespace K1s0.Sdk;

public sealed class AuditFacade
{
    private readonly K1s0Client _client;
    internal AuditFacade(K1s0Client client) { _client = client; }

    /// RecordAsync: 監査イベント記録。auditId を返す。
    /// 共通規約 §「冪等性と再試行」: idempotencyKey が空でなければ tier1 が 24h dedup
    /// する（hash chain 二重追記防止）。空文字なら毎回新 entry が作られる。
    public async Task<string> RecordAsync(
        string actor, string action, string resource, string outcome,
        IDictionary<string, string>? attributes = null,
        string idempotencyKey = "",
        CancellationToken ct = default)
    {
        var ev = new AuditEvent
        {
            Timestamp = Timestamp.FromDateTime(DateTime.UtcNow),
            Actor = actor,
            Action = action,
            Resource = resource,
            Outcome = outcome,
        };
        if (attributes != null) foreach (var kv in attributes) ev.Attributes.Add(kv.Key, kv.Value);
        var resp = await _client.Raw.Audit.RecordAsync(
            new RecordAuditRequest
            {
                Event = ev,
                IdempotencyKey = idempotencyKey,
                Context = _client.TenantContext(),
            },
            cancellationToken: ct);
        return resp.AuditId;
    }

    /// QueryAsync: 監査イベント検索。
    public async Task<IReadOnlyList<AuditEvent>> QueryAsync(
        DateTime from, DateTime to, IDictionary<string, string>? filters = null, int limit = 100, CancellationToken ct = default)
    {
        var req = new QueryAuditRequest
        {
            From = Timestamp.FromDateTime(from.ToUniversalTime()),
            To = Timestamp.FromDateTime(to.ToUniversalTime()),
            Limit = limit,
            Context = _client.TenantContext(),
        };
        if (filters != null) foreach (var kv in filters) req.Filters.Add(kv.Key, kv.Value);
        var resp = await _client.Raw.Audit.QueryAsync(req, cancellationToken: ct);
        return resp.Events.ToList();
    }

    /// ExportAsync: Audit のサーバストリーミング エクスポート（FR-T1-AUDIT-003）。
    /// 範囲 + フォーマット指定で逐次 chunk を IAsyncEnumerable&lt;ExportAuditChunk&gt; で返す。
    /// 利用例:
    ///   await foreach (var chunk in client.Audit.ExportAsync(null, null, ExportFormat.Ndjson)) { ... }
    /// from / to に null を渡すと全範囲。chunkBytes が 0 ならサーバ既定（65536）、上限は 1 MiB。
    public async IAsyncEnumerable<ExportAuditChunk> ExportAsync(
        DateTime? from = null,
        DateTime? to = null,
        ExportFormat format = ExportFormat.Ndjson,
        int chunkBytes = 0,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        var req = new ExportAuditRequest
        {
            Format = format,
            ChunkBytes = chunkBytes,
            Context = _client.TenantContext(),
        };
        if (from.HasValue) req.From = Timestamp.FromDateTime(from.Value.ToUniversalTime());
        if (to.HasValue) req.To = Timestamp.FromDateTime(to.Value.ToUniversalTime());
        using var call = _client.Raw.Audit.Export(req, cancellationToken: ct);
        while (await call.ResponseStream.MoveNext(ct))
        {
            var chunk = call.ResponseStream.Current;
            yield return chunk;
            if (chunk.IsLast) break;
        }
    }

    /// VerifyChainAsync: ハッシュチェーンの整合性検証（FR-T1-AUDIT-002）。
    /// from / to を null にすると全範囲を対象にする。
    public async Task<VerifyChainResult> VerifyChainAsync(
        DateTime? from = null, DateTime? to = null, CancellationToken ct = default)
    {
        var req = new VerifyChainRequest { Context = _client.TenantContext() };
        if (from.HasValue) req.From = Timestamp.FromDateTime(from.Value.ToUniversalTime());
        if (to.HasValue) req.To = Timestamp.FromDateTime(to.Value.ToUniversalTime());
        var resp = await _client.Raw.Audit.VerifyChainAsync(req, cancellationToken: ct);
        return new VerifyChainResult
        {
            Valid = resp.Valid,
            CheckedCount = resp.CheckedCount,
            FirstBadSequence = resp.FirstBadSequence,
            Reason = resp.Reason,
        };
    }
}

/// VerifyChain（FR-T1-AUDIT-002）の応答を SDK 利用者向けに整理した型。
public sealed class VerifyChainResult
{
    /// チェーン整合性が取れていれば true。
    public bool Valid { get; init; }
    /// 検証対象だったイベント件数。
    public long CheckedCount { get; init; }
    /// 不整合検出時、最初に失敗した sequence_number（1-based）。Valid 時は 0。
    public long FirstBadSequence { get; init; }
    /// 不整合の理由。Valid 時は空文字。
    public string Reason { get; init; } = string.Empty;
}
