// CacheImpl.cs — k1s0 tier1 Library C# 実装: ICacheClient / ICacheLock の Redis facade 実装
// 08_キャッシュ適合仕様.md §ICacheClient（OSS 中立 L3）に準拠する。
// StackExchange.Redis の ConnectionMultiplexer を L1+ ラップして公開 API に Redis 型を露出しない。
// wall-clock TTL 禁止規約に準拠して HLC CacheTtl のみを TTL 計算に使用する。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Net: EndPoint に使用する
using System.Net;
// System.Text: Encoding に使用する
using System.Text;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// StackExchange.Redis: Redis クライアント（内部のみ使用する）
using StackExchange.Redis;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// CacheImpl は ICacheClient の StackExchange.Redis facade 実装クラス。
/// Redis の IDatabase を内部に隠蔽して公開 API に Redis 型を露出しない。
/// tenant 分離を prefix で強制する（tenantId を key prefix として付与する）。
/// wall-clock TTL 禁止規約に準拠して HLC tick から TimeSpan を変換する。
/// </summary>
// CacheImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class CacheImpl : ICacheClient
{
    // _multiplexer: Redis 接続を管理する ConnectionMultiplexer（L1+ ラップのため型を隠蔽する）
    private readonly IConnectionMultiplexer _multiplexer;

    /// <summary>
    /// コンストラクタ: IConnectionMultiplexer を注入する。
    /// IConnectionMultiplexer 型で受け取り、内部でのみ参照する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: IConnectionMultiplexer を依存注入する
    public CacheImpl(IConnectionMultiplexer multiplexer)
    {
        // null チェック: null が渡された場合は例外を投げる
        _multiplexer = multiplexer ?? throw new ArgumentNullException(nameof(multiplexer));
    }

    // BuildKey は tenantId を prefix として key を構築する（tenant 分離を強制する）
    private static string BuildKey(string tenantId, string key)
    {
        // "{tenantId}:{key}" 形式で Redis キーを構築する（コロン区切りで namespace 分離する）
        return $"{tenantId}:{key}";
    }

    // TicksToTimeSpan は HLC tick 値を TimeSpan に変換する（wall-clock 禁止: HLC ベース）
    private static TimeSpan? TicksToTimeSpan(CacheTtl? ttl)
    {
        // TTL が null の場合は TimeSpan を返さない（無期限キャッシュとする）
        if (ttl is null) return null;
        // HLC tick を 100ns 単位で TimeSpan に変換する（HLC tick = .NET tick 単位）
        return TimeSpan.FromTicks((long)ttl.Value.LogicalTicks);
    }

    /// <summary>
    /// GetAsync はキーに対応する値を返す。
    /// tenantId を prefix としてテナント分離を保証する。
    /// キーが存在しない場合は null を返す。
    /// </summary>
    // GetAsync メソッド実装: Redis IDatabase.StringGetAsync を呼び出す
    public async Task<byte[]?> GetAsync(string tenantId, string key, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // tenant prefix 付きキーを構築する
        var redisKey = BuildKey(tenantId, key);
        // Redis の StringGetAsync を呼び出してバイト列を取得する
        var value = await db.StringGetAsync(redisKey).ConfigureAwait(false);
        // Redis.RedisValue が null または HasValue=false の場合は null を返す
        if (!value.HasValue) return null;
        // バイト列にキャストして返す
        return (byte[])value;
    }

    /// <summary>
    /// SetAsync はキーと値のペアをキャッシュに保存する。
    /// opts が null の場合は無期限キャッシュとして保存する。
    /// wall-clock TTL 禁止: opts.Ttl の HLC tick から TimeSpan を計算する。
    /// </summary>
    // SetAsync メソッド実装: Redis IDatabase.StringSetAsync を呼び出す
    public async Task SetAsync(string tenantId, string key, byte[] value, CacheSetOptions? opts = null, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // tenant prefix 付きキーを構築する
        var redisKey = BuildKey(tenantId, key);
        // HLC TTL を TimeSpan に変換する（null = 無期限）
        var expiry = TicksToTimeSpan(opts?.Ttl);
        // Redis SET の条件フラグを設定する（NX / XX / None）
        var when = When.Always;
        // IfNotExists が true の場合は NX フラグを設定する
        if (opts?.IfNotExists == true) when = When.NotExists;
        // IfExists が true の場合は XX フラグを設定する
        else if (opts?.IfExists == true) when = When.Exists;
        // Redis の StringSetAsync を呼び出してキャッシュに保存する
        await db.StringSetAsync(redisKey, value, expiry, when).ConfigureAwait(false);
    }

    /// <summary>
    /// DeleteAsync はキーをキャッシュから削除する（idempotent 操作）。
    /// tenantId を prefix としてテナント分離を保証する。
    /// </summary>
    // DeleteAsync メソッド実装: Redis IDatabase.KeyDeleteAsync を呼び出す
    public async Task DeleteAsync(string tenantId, string key, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // tenant prefix 付きキーを構築する
        var redisKey = BuildKey(tenantId, key);
        // Redis の KeyDeleteAsync を呼び出してキーを削除する（idempotent: 存在しなくてもエラーにしない）
        await db.KeyDeleteAsync(redisKey).ConfigureAwait(false);
    }

    /// <summary>
    /// ExistsAsync はキーが存在するかどうかを返す。
    /// tenantId を prefix としてテナント分離を保証する。
    /// </summary>
    // ExistsAsync メソッド実装: Redis IDatabase.KeyExistsAsync を呼び出す
    public async Task<bool> ExistsAsync(string tenantId, string key, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // tenant prefix 付きキーを構築する
        var redisKey = BuildKey(tenantId, key);
        // Redis の KeyExistsAsync を呼び出してキーの存在を確認する
        return await db.KeyExistsAsync(redisKey).ConfigureAwait(false);
    }

    /// <summary>
    /// GetManyAsync は複数キーの値を一括取得する（MGET 相当）。
    /// tenantId を prefix としてテナント分離を保証する。
    /// 存在しないキーは null として返す。
    /// </summary>
    // GetManyAsync メソッド実装: Redis IDatabase.StringGetAsync（複数キー）を呼び出す
    public async Task<byte[]?[]> GetManyAsync(string tenantId, IReadOnlyList<string> keys, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // 全キーに tenant prefix を付与して RedisKey 配列を構築する
        var redisKeys = new RedisKey[keys.Count];
        // 各キーに tenant prefix を付与する
        for (var i = 0; i < keys.Count; i++)
        {
            // tenant prefix 付きキーを配列に設定する
            redisKeys[i] = BuildKey(tenantId, keys[i]);
        }
        // Redis の StringGetAsync（MGET）を呼び出して一括取得する
        var values = await db.StringGetAsync(redisKeys).ConfigureAwait(false);
        // RedisValue 配列を byte[]? 配列に変換する
        var result = new byte[]?[values.Length];
        // 各 RedisValue を byte[] に変換する
        for (var i = 0; i < values.Length; i++)
        {
            // HasValue が false の場合は null を設定する
            result[i] = values[i].HasValue ? (byte[])values[i] : null;
        }
        // 変換した結果を返す
        return result;
    }

    /// <summary>
    /// SetManyAsync は複数キーと値のペアを一括保存する（MSET 相当）。
    /// tenantId を prefix としてテナント分離を保証する。
    /// opts は全ペアに適用する共通オプション。
    /// </summary>
    // SetManyAsync メソッド実装: Redis IDatabase.StringSetAsync（複数キー）を呼び出す
    public async Task SetManyAsync(string tenantId, IReadOnlyDictionary<string, byte[]> pairs, CacheSetOptions? opts = null, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // HLC TTL を TimeSpan に変換する（null = 無期限）
        var expiry = TicksToTimeSpan(opts?.Ttl);
        // 全ペアを並列に保存する（Pipeline 最適化）
        var tasks = new List<Task>(pairs.Count);
        // 各ペアを SetAsync で保存する
        foreach (var kv in pairs)
        {
            // tenant prefix 付きキーを構築する
            var redisKey = BuildKey(tenantId, kv.Key);
            // Redis の StringSetAsync を呼び出す
            tasks.Add(db.StringSetAsync(redisKey, kv.Value, expiry));
        }
        // 全タスクの完了を待機する
        await Task.WhenAll(tasks).ConfigureAwait(false);
    }

    /// <summary>
    /// DeleteManyAsync は複数キーを一括削除する（DEL 相当: idempotent 操作）。
    /// tenantId を prefix としてテナント分離を保証する。
    /// </summary>
    // DeleteManyAsync メソッド実装: Redis IDatabase.KeyDeleteAsync（複数キー）を呼び出す
    public async Task DeleteManyAsync(string tenantId, IReadOnlyList<string> keys, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // 全キーに tenant prefix を付与して RedisKey 配列を構築する
        var redisKeys = new RedisKey[keys.Count];
        // 各キーに tenant prefix を付与する
        for (var i = 0; i < keys.Count; i++)
        {
            // tenant prefix 付きキーを配列に設定する
            redisKeys[i] = BuildKey(tenantId, keys[i]);
        }
        // Redis の KeyDeleteAsync（複数キー）を呼び出して一括削除する
        await db.KeyDeleteAsync(redisKeys).ConfigureAwait(false);
    }

    /// <summary>
    /// IncrementAsync はキーの数値を delta だけアトミックに加算して新しい値を返す（INCRBY 相当）。
    /// tenantId を prefix としてテナント分離を保証する。
    /// </summary>
    // IncrementAsync メソッド実装: Redis IDatabase.StringIncrementAsync を呼び出す
    public async Task<long> IncrementAsync(string tenantId, string key, long delta, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // tenant prefix 付きキーを構築する
        var redisKey = BuildKey(tenantId, key);
        // Redis の StringIncrementAsync を呼び出してアトミックに加算する
        return await db.StringIncrementAsync(redisKey, delta).ConfigureAwait(false);
    }
}

/// <summary>
/// CacheLockImpl は ICacheLock の StackExchange.Redis 分散ロック facade 実装クラス。
/// Redis の SET NX PX を使った分散ロック（RedLock 相当）を実装する。
/// wall-clock TTL 禁止規約に準拠して HLC CacheLockOptions.Ttl のみを使用する。
/// </summary>
// CacheLockImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class CacheLockImpl : ICacheLock
{
    // _multiplexer: Redis 接続を管理する ConnectionMultiplexer（L1+ ラップ）
    private readonly IConnectionMultiplexer _multiplexer;
    // _lockKeyPrefix: ロックキーのプレフィックス（デフォルト: "lock:"）
    private const string LockKeyPrefix = "lock:";

    /// <summary>
    /// コンストラクタ: IConnectionMultiplexer を注入する。
    /// </summary>
    // コンストラクタ: IConnectionMultiplexer を依存注入する
    public CacheLockImpl(IConnectionMultiplexer multiplexer)
    {
        // null チェック: null が渡された場合は例外を投げる
        _multiplexer = multiplexer ?? throw new ArgumentNullException(nameof(multiplexer));
    }

    // BuildLockKey は tenantId と name からロックキーを構築する
    private static string BuildLockKey(string tenantId, string name)
    {
        // "{tenantId}:lock:{name}" 形式でロックキーを構築する
        return $"{tenantId}:{LockKeyPrefix}{name}";
    }

    /// <summary>
    /// AcquireAsync はロックを取得する。
    /// RetryCount 回までリトライする。
    /// 取得できなかった場合は InvalidOperationException を投げる。
    /// wall-clock 禁止: opts.Ttl の HLC tick から TimeSpan を計算する。
    /// </summary>
    // AcquireAsync メソッド実装: SET NX PX でロックを取得する
    public async Task<string> AcquireAsync(string tenantId, string name, CacheLockOptions opts, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // ロックキーを構築する
        var lockKey = BuildLockKey(tenantId, name);
        // Fencing Token として UUID v4 を生成する（SET NX の値として使用する）
        var token = Guid.NewGuid().ToString("N");
        // HLC tick から TimeSpan を変換する（wall-clock 禁止）
        var expiry = TimeSpan.FromTicks((long)opts.Ttl.LogicalTicks);
        // リトライカウント（0 = リトライなし）
        var maxRetries = Math.Max(0, opts.RetryCount);
        // ロック取得を試みるループ（最大 maxRetries + 1 回）
        for (var attempt = 0; attempt <= maxRetries; attempt++)
        {
            // Redis の SET NX PX でロックを取得する
            var acquired = await db.StringSetAsync(lockKey, token, expiry, When.NotExists).ConfigureAwait(false);
            // ロック取得成功の場合はトークンを返す
            if (acquired) return token;
            // リトライ間隔を設定する（指数バックオフは省略してシンプルな固定待機）
            if (attempt < maxRetries)
            {
                // 10ms 待機してリトライする
                await Task.Delay(10, cancellationToken).ConfigureAwait(false);
            }
        }
        // リトライ上限を超えた場合は例外を投げる
        throw new InvalidOperationException($"CacheLock.AcquireAsync: ロック取得失敗 tenant={tenantId} name={name}");
    }

    /// <summary>
    /// ReleaseAsync はロックを解放する（Fencing Token 保護）。
    /// token が不一致の場合はロックを解放せずに例外を投げる。
    /// </summary>
    // ReleaseAsync メソッド実装: Lua スクリプトで token 検証後に削除する
    public async Task ReleaseAsync(string tenantId, string name, string token, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // ロックキーを構築する
        var lockKey = BuildLockKey(tenantId, name);
        // Lua スクリプト: token が一致する場合のみ削除する（Fencing Token 保護）
        const string lua = @"
if redis.call('get', KEYS[1]) == ARGV[1] then
    return redis.call('del', KEYS[1])
else
    return 0
end";
        // Lua スクリプトを実行する（アトミック操作）
        var result = (long)await db.ScriptEvaluateAsync(lua, new RedisKey[] { lockKey }, new RedisValue[] { token }).ConfigureAwait(false);
        // result が 0 の場合は token 不一致（ロック解放失敗）
        if (result == 0)
        {
            // token 不一致の場合は例外を投げる（Fencing Token 保護）
            throw new InvalidOperationException($"CacheLock.ReleaseAsync: token 不一致または期限切れ tenant={tenantId} name={name}");
        }
    }

    /// <summary>
    /// RefreshAsync はロックの有効期限を延長する（long-running 処理用）。
    /// wall-clock 禁止: opts.Ttl の HLC tick から TimeSpan を計算する。
    /// </summary>
    // RefreshAsync メソッド実装: KeyExpireAsync でロック有効期限を延長する
    public async Task RefreshAsync(string tenantId, string name, string token, CacheLockOptions opts, CancellationToken cancellationToken = default)
    {
        // Redis データベース接続を取得する
        var db = _multiplexer.GetDatabase();
        // ロックキーを構築する
        var lockKey = BuildLockKey(tenantId, name);
        // HLC tick から TimeSpan を変換する（wall-clock 禁止）
        var expiry = TimeSpan.FromTicks((long)opts.Ttl.LogicalTicks);
        // Lua スクリプト: token が一致する場合のみ有効期限を延長する
        const string lua = @"
if redis.call('get', KEYS[1]) == ARGV[1] then
    return redis.call('pexpire', KEYS[1], ARGV[2])
else
    return 0
end";
        // 有効期限をミリ秒に変換する
        var expiryMs = (long)expiry.TotalMilliseconds;
        // Lua スクリプトを実行する（アトミック操作）
        var result = (long)await db.ScriptEvaluateAsync(lua, new RedisKey[] { lockKey }, new RedisValue[] { token, expiryMs }).ConfigureAwait(false);
        // result が 0 の場合は token 不一致（期限延長失敗）
        if (result == 0)
        {
            // token 不一致の場合は例外を投げる
            throw new InvalidOperationException($"CacheLock.RefreshAsync: token 不一致または期限切れ tenant={tenantId} name={name}");
        }
    }
}
