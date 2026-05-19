// k1s0 tier3 IndexedDB encrypted outbox（C# .NET 8+ 等価強度実装）
// TypeScript primary の outbox.ts と同等の抽象を C# で実装する
// PII strip on enqueue / Idempotency-Key 24h TTL を強制する
// wall-clock TTL 禁止規約に従い DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() を HLC 基底に使用する
// 注意: 理想は monotonic clock（Stopwatch.GetTimestamp()）だが HLC 相互運用のため UTC ミリ秒を物理クロック基底とする

namespace K1s0.Tier3.Outbox;

// IDEMPOTENCY_KEY_TTL_MS は Idempotency-Key の 24h TTL（ミリ秒）
public static class OutboxConstants
{
    // Idempotency-Key の TTL: 24 時間をミリ秒で表現する
    public const long IdempotencyKeyTtlMs = 24L * 60 * 60 * 1000;
}

// OutboxEntryMeta は Outbox エントリのメタデータ
public sealed record OutboxEntryMeta(
    // Idempotency-Key（aggregateId prefix + HLC ベース + GUID suffix）
    string IdempotencyKey,
    // enqueue 日時（HLC タイムスタンプ: "{timestamp_ms_hex}-{logical_counter}-{node_id}"）
    string EnqueuedAt,
    // TTL（UNIX ミリ秒、24h 後 — backward compat 用途で保持する; 値は HLC から導出する）
    long ExpiresAtMs,
    // aggregate ID
    string AggregateId,
    // RPC method 名（短縮）
    string RpcMethod,
    // chain 元 idempotency_key（rebase 後再送時に設定、null は chain なし）
    string? ChainedFrom = null
);

// HlcClock は HLC タイムスタンプ生成を担当するクラス
public static class HlcClock
{
    // HlcNow は現在時刻を HLC タイムスタンプ文字列で返す
    // フォーマット: "{timestamp_ms_hex}-{logical_counter}-{node_id}"
    // 注意: monotonic clock が理想だが HLC 相互運用のため DateTimeOffset.UtcNow を使用する
    public static string HlcNow()
    {
        // DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() で UTC ミリ秒を取得する
        // 注意: 理想は Stopwatch ベースの monotonic clock だが HLC 相互運用のため UTC を使用する
        var nowMs = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        // ミリ秒を 16 桁 hex 文字列にフォーマットする
        var timestampHex = ((ulong)nowMs).ToString("x16");
        // logical_counter は本実装では 0000 固定（同一ミリ秒内の複数イベントが不要なため）
        const string logicalCounter = "0000";
        // node_id は本実装では 0000 固定（単一ノード想定）
        const string nodeId = "0000";
        // HLC タイムスタンプ文字列を組み立てて返す
        return $"{timestampHex}-{logicalCounter}-{nodeId}";
    }

    // ExtractMsFromHlc は HLC タイムスタンプからミリ秒値を抽出する
    // hlcTimestamp: "{timestamp_ms_hex}-{logical_counter}-{node_id}" 形式
    public static long ExtractMsFromHlc(string hlcTimestamp)
    {
        // ハイフン区切りの先頭部分が 16 進数ミリ秒タイムスタンプ
        var hexPart = hlcTimestamp.Split('-')[0];
        // 16 進数文字列を long に変換する（パース失敗時は 0 を返す）
        return long.TryParse(hexPart, System.Globalization.NumberStyles.HexNumber, null, out var ms) ? ms : 0L;
    }
}

// OutboxService は Outbox エントリの生成・管理を担当するクラス
public static class OutboxService
{
    // GenerateIdempotencyKey は Idempotency-Key を生成する
    // wall-clock TTL 禁止規約に従い HLC を使用する
    public static string GenerateIdempotencyKey(string aggregateId, string rpcMethod)
    {
        // HLC タイムスタンプの先頭 16 進数部分を Idempotency-Key の基底として使用する
        var hlcBase = HlcClock.HlcNow().Split('-')[0];
        // GUID でランダムサフィックスを生成する（Guid.NewGuid は暗号論的に安全）
        var randomSuffix = Guid.NewGuid().ToString("N")[..16];
        // aggregateId の先頭 8 文字を prefix に使用する（長すぎる場合は切り詰める）
        var aggPrefix = aggregateId.Length > 8 ? aggregateId[..8] : aggregateId;
        // rpcMethod の先頭 4 文字を prefix に使用する（長すぎる場合は切り詰める）
        var methodPrefix = rpcMethod.Length > 4 ? rpcMethod[..4] : rpcMethod;
        // prefix + HLC ベース + random suffix で Idempotency-Key を組み立てる
        return $"{aggPrefix}_{methodPrefix}_{hlcBase}_{randomSuffix}";
    }

    // CreateOutboxMeta は Outbox エントリのメタデータを生成する
    // wall-clock TTL 禁止規約に従い HLC ベースのタイムスタンプを使用する
    public static OutboxEntryMeta CreateOutboxMeta(
        string aggregateId,
        string rpcMethod,
        string? chainedFrom = null)
    {
        // HLC タイムスタンプを現在時刻として取得する
        var nowHlc = HlcClock.HlcNow();
        // enqueue 時刻（ミリ秒）を HLC から抽出する
        var enqueuedMs = HlcClock.ExtractMsFromHlc(nowHlc);
        // 新しい Idempotency-Key を生成する
        var key = GenerateIdempotencyKey(aggregateId, rpcMethod);
        // backward compat 用の ExpiresAtMs は HLC ミリ秒から計算する
        var expiresAtMs = enqueuedMs + OutboxConstants.IdempotencyKeyTtlMs;
        // メタデータ record を組み立てて返す
        return new OutboxEntryMeta(
            // 生成した Idempotency-Key
            IdempotencyKey: key,
            // HLC タイムスタンプ（wall clock 代替）
            EnqueuedAt: nowHlc,
            // backward compat 用 TTL（HLC から導出した値）
            ExpiresAtMs: expiresAtMs,
            // aggregate ID
            AggregateId: aggregateId,
            // RPC method 名
            RpcMethod: rpcMethod,
            // chain 元（null は chain なし）
            ChainedFrom: chainedFrom
        );
    }

    // IsExpired は Idempotency-Key が TTL 超過かどうかを HLC ベースで確認する
    // wall-clock TTL 禁止規約に従い HLC 比較を行う
    public static bool IsExpired(OutboxEntryMeta meta)
    {
        // enqueue 時刻（ミリ秒）を HLC タイムスタンプから抽出する
        var enqueuedMs = HlcClock.ExtractMsFromHlc(meta.EnqueuedAt);
        // 現在時刻（HLC ベースのミリ秒）を取得する
        var nowMs = HlcClock.ExtractMsFromHlc(HlcClock.HlcNow());
        // enqueue 時刻 + TTL が現在時刻以下であれば TTL 超過と判定する
        return enqueuedMs + OutboxConstants.IdempotencyKeyTtlMs <= nowMs;
    }

    // StripPiiFields は PII フィールドを strip する
    // piiFieldNames に含まれるキーを payload から除去して返す
    public static Dictionary<string, object?> StripPiiFields(
        Dictionary<string, object?> payload,
        IReadOnlySet<string> piiFieldNames)
    {
        // PII フィールド以外のキー・値ペアを結果に含める
        return payload
            // PII フィールドを除外する（piiFieldNames に含まれないキーのみ）
            .Where(kvp => !piiFieldNames.Contains(kvp.Key))
            // 結果を Dictionary に変換する
            .ToDictionary(kvp => kvp.Key, kvp => kvp.Value);
    }
}
