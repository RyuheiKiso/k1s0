// ICache.cs — k1s0 tier1 Library C# 実装: KeyValue / Cache の L3 interface
// 08_キャッシュ適合仕様.md §ICacheClient（OSS 中立 L3）に準拠する。
// Redis / Memcached 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
// wall-clock TTL 禁止規約に準拠して HLC CacheTtl のみを TTL として受け付ける。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// CacheTtl は HLC ベースの TTL を宣言する型。
/// wall-clock TTL 禁止規約に準拠して論理クロック差分 (LogicalTicks) で表現する。
/// 実装側は HLC の hlc_lib を参照して変換する（DateTime.UtcNow 等の直接使用を禁止する）。
/// </summary>
// CacheTtl レコード定義
public readonly record struct CacheTtl(
    // LogicalTicks: HLC 論理クロック差分（tick 単位）
    ulong LogicalTicks
);

/// <summary>
/// CacheSetOptions は ICacheClient.SetAsync / ICacheClient.SetManyAsync に渡すオプションを宣言する型。
/// </summary>
// CacheSetOptions クラス定義
public sealed class CacheSetOptions
{
    /// <summary>
    /// Ttl: HLC ベースのキャッシュ有効期限（null = 無期限キャッシュ）。
    /// wall-clock TTL 禁止規約に準拠して HLC Duration のみを許容する。
    /// </summary>
    // Ttl プロパティ
    public CacheTtl? Ttl { get; init; }

    /// <summary>IfNotExists: true の場合はキーが存在しない場合のみ設定する（SET NX 相当）</summary>
    // IfNotExists プロパティ
    public bool IfNotExists { get; init; }

    /// <summary>IfExists: true の場合はキーが既に存在する場合のみ更新する（SET XX 相当）</summary>
    // IfExists プロパティ
    public bool IfExists { get; init; }
}

/// <summary>
/// ICacheClient は KeyValue / Cache の L3 抽象 interface を宣言する。
/// Redis / Memcached / DragonflyDB 等 OSS を透過的に切り替え可能にする。
/// OSS 型（StackExchange.Redis.IDatabase 等）を引数・戻り値に一切含まない。
/// </summary>
// ICacheClient インターフェース定義
public interface ICacheClient
{
    /// <summary>
    /// GetAsync はキーに対応する値を返す。
    /// キーが存在しない場合は null を返す（エラーと区別する）。
    /// tenantId を prefix とした名前空間で tenant 分離を保証する。
    /// </summary>
    // GetAsync メソッド: キーの値を取得する
    Task<byte[]?> GetAsync(string tenantId, string key, CancellationToken cancellationToken = default);

    /// <summary>
    /// SetAsync はキーと値のペアをキャッシュに保存する。
    /// opts が null の場合は無期限キャッシュとして保存する。
    /// </summary>
    // SetAsync メソッド: キーと値のペアを保存する
    Task SetAsync(string tenantId, string key, byte[] value, CacheSetOptions? opts = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// DeleteAsync はキーをキャッシュから削除する（idempotent 操作）。
    /// </summary>
    // DeleteAsync メソッド: キーを削除する
    Task DeleteAsync(string tenantId, string key, CancellationToken cancellationToken = default);

    /// <summary>
    /// ExistsAsync はキーが存在するかどうかを返す。
    /// </summary>
    // ExistsAsync メソッド: キーの存在確認
    Task<bool> ExistsAsync(string tenantId, string key, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetManyAsync は複数キーの値を一括取得する（MGET 相当）。
    /// 戻り値は keys と同順の配列で、存在しないキーは null とする。
    /// </summary>
    // GetManyAsync メソッド: 複数キーの値を一括取得する
    Task<byte[]?[]> GetManyAsync(string tenantId, IReadOnlyList<string> keys, CancellationToken cancellationToken = default);

    /// <summary>
    /// SetManyAsync は複数キーと値のペアを一括保存する（MSET 相当）。
    /// pairs はキーと値のペア辞書、opts は全ペアに適用する共通オプション。
    /// </summary>
    // SetManyAsync メソッド: 複数キーと値のペアを一括保存する
    Task SetManyAsync(string tenantId, IReadOnlyDictionary<string, byte[]> pairs, CacheSetOptions? opts = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// DeleteManyAsync は複数キーを一括削除する（DEL 相当: idempotent 操作）。
    /// </summary>
    // DeleteManyAsync メソッド: 複数キーを一括削除する
    Task DeleteManyAsync(string tenantId, IReadOnlyList<string> keys, CancellationToken cancellationToken = default);

    /// <summary>
    /// IncrementAsync はキーの数値を delta だけアトミックに加算して新しい値を返す（INCRBY 相当）。
    /// </summary>
    // IncrementAsync メソッド: キーの数値をアトミックに加算する
    Task<long> IncrementAsync(string tenantId, string key, long delta, CancellationToken cancellationToken = default);
}

/// <summary>
/// CacheLockOptions は分散ロック（RedLock / Fencing Token）のオプションを宣言する型。
/// wall-clock TTL 禁止規約に準拠して HLC ベースのロック有効期限のみを受け付ける。
/// </summary>
// CacheLockOptions クラス定義
public sealed class CacheLockOptions
{
    /// <summary>Ttl: HLC ベースのロック有効期限（必須: 無期限ロックは禁止する）</summary>
    // Ttl プロパティ（必須）
    public required CacheTtl Ttl { get; init; }

    /// <summary>RetryCount: ロック取得リトライ回数（0 = リトライなし）</summary>
    // RetryCount プロパティ
    public int RetryCount { get; init; }

    /// <summary>FencingToken: Fencing Token 機能を有効にするかどうか（stale write 防止）</summary>
    // FencingToken プロパティ
    public bool FencingToken { get; init; }
}

/// <summary>
/// ICacheLock はキャッシュ分散ロックの L3 抽象 interface を宣言する。
/// RedLock / Redisson 等を隠蔽する。
/// </summary>
// ICacheLock インターフェース定義
public interface ICacheLock
{
    /// <summary>
    /// AcquireAsync はロックを取得する（取得できなかった場合はエラーを投げる）。
    /// 戻り値は Fencing Token として使用するロックトークン。
    /// </summary>
    // AcquireAsync メソッド: ロックを取得する
    Task<string> AcquireAsync(string tenantId, string name, CacheLockOptions opts, CancellationToken cancellationToken = default);

    /// <summary>
    /// ReleaseAsync はロックを解放する（token は AcquireAsync で返されたトークン）。
    /// token が不一致の場合はロックを解放せずにエラーを投げる（Fencing Token 保護）。
    /// </summary>
    // ReleaseAsync メソッド: ロックを解放する
    Task ReleaseAsync(string tenantId, string name, string token, CancellationToken cancellationToken = default);

    /// <summary>
    /// RefreshAsync はロックの有効期限を延長する（long-running 処理用）。
    /// </summary>
    // RefreshAsync メソッド: ロックの有効期限を延長する
    Task RefreshAsync(string tenantId, string name, string token, CacheLockOptions opts, CancellationToken cancellationToken = default);
}
