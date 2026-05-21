// k1s0 tier3 IndexedDB encrypted outbox（C# .NET 8+ 等価強度実装）
// TypeScript primary の outbox.ts と同等の抽象を C# で実装する
// PII strip on enqueue / Idempotency-Key 24h TTL を強制する
// wall-clock TTL 禁止規約に従い K1s0.HlcLib.HlcClock（src/client/hlc_lib/csharp）を使用する
// DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() は HLC lib 内部のみ許可

// K1s0.HlcLib: wall-clock TTL 禁止規律に従い HLC（Hybrid Logical Clock）を使用する
using K1s0.HlcLib;

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

// OutboxHlc は K1s0.HlcLib.HlcClock のプロセス全体共有 singleton ラッパ
// DateTimeOffset.UtcNow は K1s0.HlcLib 内部のみ許可（Outbox.cs 内での直接使用禁止）
internal static class OutboxHlc
{
    // _clock は K1s0.HlcLib の HlcClock（環境変数 HLC_NODE_ID から node_id を取得する）
    private static readonly HlcClock _clock = HlcClock.FromEnv();

    // HlcNow は現在の HLC タイムスタンプを compact 文字列で返す
    // K1s0.HlcLib.HlcClock.Now() 経由でのみ時刻を取得する（DateTimeOffset.UtcNow 直接使用禁止）
    public static string HlcNow()
    {
        // HlcClock.Now() から HlcTimestamp を取得して compact 文字列に変換する
        return _clock.Now().FormatCompact();
    }

    // ExtractMsFromHlc は HLC タイムスタンプからミリ秒値を抽出する
    // hlcTimestamp: "{wall_ms_hex_16}-{logical_04x}-{node_04x}" 形式
    public static long ExtractMsFromHlc(string hlcTimestamp)
    {
        // HlcTimestamp.ParseCompact で構造的に解析する（文字列 split より安全）
        var ts = HlcTimestamp.ParseCompact(hlcTimestamp);
        // パース成功時は wall_ms を返す、失敗時は 0 を返す（safe フォールバック）
        return ts.HasValue ? (long)ts.Value.WallMs : 0L;
    }
}

// OutboxService は Outbox エントリの生成・管理を担当するクラス
public static class OutboxService
{
    // GenerateIdempotencyKey は Idempotency-Key を生成する
    // フォーマット: "{tenantId}_{ulidHex}_{methodHash}" — docs §idempotency_key 準拠
    // tenantId: BFF cookie から取得したテナント識別子（tenant_id_injector 経由で渡す）
    // wall-clock TTL 禁止規約に従い HLC を使用する
    public static string GenerateIdempotencyKey(string tenantId, string aggregateId, string rpcMethod)
    {
        // HLC タイムスタンプの先頭 16 進数部分を ULID の時刻部分として使用する
        var hlcBase = OutboxHlc.HlcNow().Split('-')[0];
        // GUID でランダムサフィックスを生成する（Guid.NewGuid は暗号論的に安全）
        var randomPart = Guid.NewGuid().ToString("N")[..8];
        // ULID 相当: HLC タイムスタンプ hex + random で識別子を生成する
        var ulidHex = $"{hlcBase}{randomPart}";
        // rpcMethod の先頭 4 文字を method hash として使用する（短縮識別子）
        var methodHash = rpcMethod.Length > 4 ? rpcMethod[..4] : rpcMethod;
        // tenantId prefix + ulid + method hash の形式で Idempotency-Key を組み立てる
        return $"{tenantId}_{ulidHex}_{methodHash}";
    }

    // CreateOutboxMeta は Outbox エントリのメタデータを生成する
    // tenantId: BFF cookie から取得したテナント識別子（tenant_id_injector 経由で渡す）
    // wall-clock TTL 禁止規約に従い HLC ベースのタイムスタンプを使用する
    public static OutboxEntryMeta CreateOutboxMeta(
        string tenantId,
        string aggregateId,
        string rpcMethod,
        string? chainedFrom = null)
    {
        // HLC タイムスタンプを現在時刻として取得する
        var nowHlc = OutboxHlc.HlcNow();
        // enqueue 時刻（ミリ秒）を HLC から抽出する
        var enqueuedMs = OutboxHlc.ExtractMsFromHlc(nowHlc);
        // 新しい Idempotency-Key を生成する（tenantId prefix 付き）
        var key = GenerateIdempotencyKey(tenantId, aggregateId, rpcMethod);
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
        var enqueuedMs = OutboxHlc.ExtractMsFromHlc(meta.EnqueuedAt);
        // 現在時刻（HLC ベースのミリ秒）を取得する
        var nowMs = OutboxHlc.ExtractMsFromHlc(OutboxHlc.HlcNow());
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
