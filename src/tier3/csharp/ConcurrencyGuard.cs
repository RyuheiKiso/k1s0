// ConcurrencyGuard.cs — aggregate_id 単位で max_one_in_flight を強制する ConcurrencyGuard（C# .NET 8+ 等価強度実装）
// TypeScript の withAggregateExclusivity と等価の抽象を C# で実装する
// spec 11 §per-aggregate write 並行 1 件以下: layers.yaml invariant の max_one_in_flight_per_aggregate を物理化する

namespace K1s0.Tier3.State;

// ConcurrencyGuard は aggregate_id 単位で max_one_in_flight を実現するシールドクラス
// ConcurrentDictionary<string, SemaphoreSlim> で aggregate 単位の Semaphore を管理する
// TypeScript の inFlightMap（Map<string, Promise<void>>）に対応する実装
public sealed class ConcurrencyGuard
{
    // aggregate_id → SemaphoreSlim のマップ（ConcurrentDictionary で thread-safe に管理する）
    // SemaphoreSlim(1,1) は max_one_in_flight_per_aggregate を実現する（binary semaphore 相当）
    private readonly System.Collections.Concurrent.ConcurrentDictionary<string, SemaphoreSlim>
        _semaphores = new();

    // WithAggregateExclusivityAsync は aggregateId で指定した aggregate に対して fn を排他的に実行する
    // 先行処理が存在する場合は先行処理の完了まで待機してから fn を実行する（TypeScript と等価）
    // aggregateId: 排他制御の対象 aggregate の識別子
    // fn: aggregate に対して排他的に実行する非同期処理（Task<T> を返す）
    // cancellationToken: キャンセルトークン（オプション）
    public async Task<T> WithAggregateExclusivityAsync<T>(
        // 排他制御の対象 aggregate の識別子を受け取る
        string aggregateId,
        // 排他的に実行する非同期処理（Task<T> を返す Func）を受け取る
        Func<Task<T>> fn,
        // キャンセルトークン（デフォルトは CancellationToken.None）
        CancellationToken cancellationToken = default
    )
    {
        // aggregate_id に対応する SemaphoreSlim を取得または生成する（binary semaphore: 初期値 1 / 最大値 1）
        // GetOrAdd は thread-safe に既存エントリを取得するか新規生成する
        var semaphore = _semaphores.GetOrAdd(aggregateId, _ => new SemaphoreSlim(1, 1));
        // SemaphoreSlim を取得する（先行処理が完了するまで待機する）
        // TypeScript の await current.catch(() => undefined) + inFlightMap.set に対応する
        await semaphore.WaitAsync(cancellationToken).ConfigureAwait(false);
        // fn 完了後に SemaphoreSlim を解放する（次の待機者を解放する）
        // try/finally で確実に Release する（例外 / キャンセルでも解放する）
        try
        {
            // 排他的な非同期処理を実行して結果を返す
            return await fn().ConfigureAwait(false);
        }
        finally
        {
            // SemaphoreSlim を解放する（次の待機者を解放する）
            semaphore.Release();
        }
    }

    // WithAggregateExclusivityAsync の void オーバーロード（戻り値なしの処理向け）
    // aggregateId: 排他制御の対象 aggregate の識別子
    // fn: aggregate に対して排他的に実行する非同期処理（Task を返す）
    // cancellationToken: キャンセルトークン（オプション）
    public async Task WithAggregateExclusivityAsync(
        // 排他制御の対象 aggregate の識別子を受け取る
        string aggregateId,
        // 排他的に実行する非同期処理（Task を返す Func）を受け取る
        Func<Task> fn,
        // キャンセルトークン（デフォルトは CancellationToken.None）
        CancellationToken cancellationToken = default
    )
    {
        // Task<T> オーバーロードに委譲する（戻り値を Unit で代替する）
        await WithAggregateExclusivityAsync<int>(
            // aggregate の識別子を渡す
            aggregateId,
            // void fn を Task<int> に wrap する
            async () => { await fn().ConfigureAwait(false); return 0; },
            // キャンセルトークンを渡す
            cancellationToken
        ).ConfigureAwait(false);
    }

    // IsAggregateInFlight は aggregateId で指定した aggregate が現在 in-flight かどうかを返す
    // TypeScript の isAggregateInFlight と等価の確認関数
    // aggregateId: 確認対象 aggregate の識別子
    public bool IsAggregateInFlight(string aggregateId)
    {
        // _semaphores に aggregateId のエントリが存在かつ CurrentCount が 0（占有中）かを確認する
        if (_semaphores.TryGetValue(aggregateId, out var semaphore))
        {
            // SemaphoreSlim.CurrentCount が 0 の場合は in-flight（処理占有中）
            return semaphore.CurrentCount == 0;
        }
        // エントリが存在しない場合は in-flight でない
        return false;
    }
}
