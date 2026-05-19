// DbDistributedImpl.cs — k1s0 tier1 Library C# 実装: IDistributedDbClient / IDistributedDbTx の CockroachDB facade 実装
// 14_分散SQL適合仕様.md §IDistributedDbClient（CockroachDB / Spanner L1+ 深耕）に準拠する。
// Npgsql の CockroachDB 互換接続を L1+ ラップして公開 API に Npgsql 型を露出しない。
// wall-clock TTL 禁止: AsOfSystemTimeTick は HLC tick で指定する。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyList に使用する
using System.Collections.Generic;
// System.Data: IsolationLevel に使用する
using System.Data;
// System.Linq: Dapper の拡張メソッドに使用する
using System.Linq;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// Npgsql: PostgreSQL / CockroachDB クライアント（内部のみ使用する）
using Npgsql;
// Dapper: 軽量 ORM（クエリマッピングに使用する）
using Dapper;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// DistributedDbTxImpl は IDistributedDbTx の Npgsql CockroachDB facade 実装クラス。
/// IDbTx を継承しつつ、Savepoint 等の CockroachDB 固有メソッドを追加する。
/// NpgsqlTransaction を内部に隠蔽して公開 API に Npgsql 型を露出しない。
/// </summary>
// DistributedDbTxImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class DistributedDbTxImpl : IDistributedDbTx
{
    // _conn: Npgsql 接続（internal でのみ参照する）
    private readonly NpgsqlConnection _conn;
    // _tx: Npgsql トランザクション（内部に隠蔽する）
    private readonly NpgsqlTransaction _tx;
    // _priority: トランザクション優先度（CockroachDB 固有）
    private readonly DistributedTxPriority _priority;
    // _disposed: 二重 Dispose 防止フラグ
    private bool _disposed;

    /// <summary>
    /// コンストラクタ: NpgsqlConnection / NpgsqlTransaction / priority を受け取る。
    /// </summary>
    // コンストラクタ: 接続 / トランザクション / 優先度を受け取る
    internal DistributedDbTxImpl(NpgsqlConnection conn, NpgsqlTransaction tx, DistributedTxPriority priority)
    {
        // 接続を保持する
        _conn = conn;
        // トランザクションを保持する
        _tx = tx;
        // 優先度を保持する
        _priority = priority;
        // 初期状態は未 Dispose
        _disposed = false;
    }

    /// <summary>Priority は現在のトランザクション優先度を返す。</summary>
    // Priority プロパティ実装
    public DistributedTxPriority Priority => _priority;

    /// <summary>
    /// QuerySingleOrDefaultAsync は単一行クエリを実行して結果を返す。
    /// </summary>
    // QuerySingleOrDefaultAsync メソッド実装: Dapper を通じてクエリを実行する
    public async Task<T?> QuerySingleOrDefaultAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // Dapper の QuerySingleOrDefaultAsync を呼び出す（トランザクション内で実行する）
        return await _conn.QuerySingleOrDefaultAsync<T>(query, param, _tx).ConfigureAwait(false);
    }

    /// <summary>
    /// QueryAsync は複数行クエリを実行して結果リストを返す。
    /// </summary>
    // QueryAsync メソッド実装: Dapper を通じてクエリを実行する
    public async Task<IReadOnlyList<T>> QueryAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // Dapper の QueryAsync を呼び出す
        var rows = await _conn.QueryAsync<T>(query, param, _tx).ConfigureAwait(false);
        // IEnumerable を IReadOnlyList に変換して返す
        return rows.ToList().AsReadOnly();
    }

    /// <summary>
    /// ExecuteAsync は DML / DDL を実行して変更された行数を返す。
    /// </summary>
    // ExecuteAsync メソッド実装: Dapper を通じて DML を実行する
    public async Task<int> ExecuteAsync(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // Dapper の ExecuteAsync を呼び出す
        return await _conn.ExecuteAsync(query, param, _tx).ConfigureAwait(false);
    }

    /// <summary>
    /// CommitAsync はトランザクションをコミットする。
    /// </summary>
    // CommitAsync メソッド実装: NpgsqlTransaction.CommitAsync を呼び出す
    public async Task CommitAsync(CancellationToken cancellationToken = default)
    {
        // Npgsql トランザクションをコミットする
        await _tx.CommitAsync(cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// RollbackAsync はトランザクションをロールバックする。
    /// </summary>
    // RollbackAsync メソッド実装: NpgsqlTransaction.RollbackAsync を呼び出す
    public async Task RollbackAsync(CancellationToken cancellationToken = default)
    {
        // Npgsql トランザクションをロールバックする
        await _tx.RollbackAsync(cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// SavepointAsync は名前付きセーブポイントを作成する（CockroachDB の SAVEPOINT 相当）。
    /// </summary>
    // SavepointAsync メソッド実装: SAVEPOINT SQL を実行する
    public async Task SavepointAsync(string name, CancellationToken cancellationToken = default)
    {
        // SAVEPOINT SQL を実行する（CockroachDB の SAVEPOINT に対応する）
        await _conn.ExecuteAsync($"SAVEPOINT {name}", transaction: _tx).ConfigureAwait(false);
    }

    /// <summary>
    /// RollbackToSavepointAsync はセーブポイントにロールバックする。
    /// </summary>
    // RollbackToSavepointAsync メソッド実装: ROLLBACK TO SAVEPOINT SQL を実行する
    public async Task RollbackToSavepointAsync(string name, CancellationToken cancellationToken = default)
    {
        // ROLLBACK TO SAVEPOINT SQL を実行する
        await _conn.ExecuteAsync($"ROLLBACK TO SAVEPOINT {name}", transaction: _tx).ConfigureAwait(false);
    }

    /// <summary>
    /// ReleaseSavepointAsync はセーブポイントをリリースする（確定する）。
    /// </summary>
    // ReleaseSavepointAsync メソッド実装: RELEASE SAVEPOINT SQL を実行する
    public async Task ReleaseSavepointAsync(string name, CancellationToken cancellationToken = default)
    {
        // RELEASE SAVEPOINT SQL を実行する
        await _conn.ExecuteAsync($"RELEASE SAVEPOINT {name}", transaction: _tx).ConfigureAwait(false);
    }

    /// <summary>
    /// DisposeAsync は await using での自動ロールバックを実現する。
    /// </summary>
    // DisposeAsync メソッド実装: IAsyncDisposable の実装
    public async ValueTask DisposeAsync()
    {
        // 二重 Dispose を防止する
        if (_disposed) return;
        // Dispose フラグを立てる
        _disposed = true;
        // トランザクションをロールバックする（Commit 済みの場合は no-op）
        try
        {
            // RollbackAsync を呼び出す（既に Commit 済みの場合はエラーを無視する）
            await _tx.RollbackAsync().ConfigureAwait(false);
        }
        catch (InvalidOperationException)
        {
            // Commit 済みの場合は InvalidOperationException が発生するが無視する
        }
        // NpgsqlTransaction を Dispose する
        await _tx.DisposeAsync().ConfigureAwait(false);
        // NpgsqlConnection を Dispose する
        await _conn.DisposeAsync().ConfigureAwait(false);
    }
}

/// <summary>
/// DbDistributedImpl は IDistributedDbClient の CockroachDB / Npgsql facade 実装クラス。
/// NpgsqlDataSource を L1+ ラップして公開 API に Npgsql 型を露出しない。
/// CockroachDB の transaction priority / AOST 等の固有機能を Library 独自語彙で提供する。
/// </summary>
// DbDistributedImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class DbDistributedImpl : IDistributedDbClient
{
    // _dataSource: Npgsql データソース（CockroachDB 接続に使用する）
    private readonly NpgsqlDataSource _dataSource;

    /// <summary>
    /// コンストラクタ: NpgsqlDataSource を注入する。
    /// CockroachDB の接続文字列を使用した NpgsqlDataSource を受け取る。
    /// </summary>
    // コンストラクタ: NpgsqlDataSource を依存注入する
    public DbDistributedImpl(NpgsqlDataSource dataSource)
    {
        // null チェック: null が渡された場合は例外を投げる
        _dataSource = dataSource ?? throw new ArgumentNullException(nameof(dataSource));
    }

    // ToPriorityString は DistributedTxPriority を CockroachDB の SQL 文字列に変換する
    private static string ToPriorityString(DistributedTxPriority priority)
    {
        // 各優先度を CockroachDB の SQL 文字列にマッピングする
        return priority switch
        {
            // High: PRIORITY HIGH
            DistributedTxPriority.High => "PRIORITY HIGH",
            // Low: PRIORITY LOW
            DistributedTxPriority.Low => "PRIORITY LOW",
            // Normal: PRIORITY NORMAL（デフォルト）
            _ => "PRIORITY NORMAL",
        };
    }

    /// <summary>
    /// BeginAsync はトランザクションを開始して IDistributedDbTx を返す。
    /// opts の priority を CockroachDB の BEGIN ... PRIORITY に適用する。
    /// opts の AsOfSystemTimeTick が 0 でない場合は AOST を設定する（wall-clock 禁止: HLC tick）。
    /// </summary>
    // BeginAsync メソッド実装: トランザクションを開始して CockroachDB 固有オプションを適用する
    public async Task<IDistributedDbTx> BeginAsync(DistributedTxOptions? opts = null, CancellationToken cancellationToken = default)
    {
        // NpgsqlDataSource から新しい接続を取得する
        var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // 分離レベルを設定する（Distributed SQL では Serializable 推奨）
        var isoLevel = (opts?.IsoLevel ?? DistributedTxOptions_DefaultIsoLevel()) switch
        {
            // ReadCommitted: System.Data.IsolationLevel.ReadCommitted
            DbTxIsoLevel.ReadCommitted => IsolationLevel.ReadCommitted,
            // Serializable: System.Data.IsolationLevel.Serializable
            DbTxIsoLevel.Serializable => IsolationLevel.Serializable,
            // RepeatableRead: System.Data.IsolationLevel.RepeatableRead
            _ => IsolationLevel.RepeatableRead,
        };
        // トランザクションを開始する
        var tx = await conn.BeginTransactionAsync(isoLevel, cancellationToken).ConfigureAwait(false);
        // 優先度を設定する（CockroachDB 固有）
        var priority = opts?.Priority ?? DistributedTxPriority.Normal;
        // SET TRANSACTION PRIORITY を実行する
        await conn.ExecuteAsync($"SET TRANSACTION {ToPriorityString(priority)}", transaction: tx).ConfigureAwait(false);
        // AsOfSystemTimeTick が 0 でない場合は AOST を設定する（wall-clock 禁止: HLC tick）
        if (opts?.AsOfSystemTimeTick > 0)
        {
            // HLC tick を CockroachDB の FOLLOWER_READ_TIMESTAMP() に変換する
            await conn.ExecuteAsync($"SET TRANSACTION AS OF SYSTEM TIME {opts.AsOfSystemTimeTick}", transaction: tx).ConfigureAwait(false);
        }
        // DistributedDbTxImpl を生成して返す
        return new DistributedDbTxImpl(conn, tx, priority);
    }

    // DistributedTxOptions_DefaultIsoLevel は DistributedTxOptions のデフォルト分離レベルを返す
    private static DbTxIsoLevel DistributedTxOptions_DefaultIsoLevel()
    {
        // デフォルトは Serializable（Distributed SQL 推奨）
        return DbTxIsoLevel.Serializable;
    }

    /// <summary>
    /// QuerySingleOrDefaultAsync は単一行クエリを autocommit モードで実行する。
    /// </summary>
    // QuerySingleOrDefaultAsync メソッド実装: autocommit モードでクエリを実行する
    public async Task<T?> QuerySingleOrDefaultAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // autocommit 用の接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // Dapper の QuerySingleOrDefaultAsync を呼び出す
        return await conn.QuerySingleOrDefaultAsync<T>(query, param).ConfigureAwait(false);
    }

    /// <summary>
    /// QueryAsync は複数行クエリを autocommit モードで実行する。
    /// </summary>
    // QueryAsync メソッド実装: autocommit モードでクエリを実行する
    public async Task<IReadOnlyList<T>> QueryAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // autocommit 用の接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // Dapper の QueryAsync を呼び出す
        var rows = await conn.QueryAsync<T>(query, param).ConfigureAwait(false);
        // IEnumerable を IReadOnlyList に変換して返す
        return rows.ToList().AsReadOnly();
    }

    /// <summary>
    /// ExecuteAsync は DML を autocommit モードで実行する。
    /// </summary>
    // ExecuteAsync メソッド実装: autocommit モードで DML を実行する
    public async Task<int> ExecuteAsync(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // autocommit 用の接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // Dapper の ExecuteAsync を呼び出す
        return await conn.ExecuteAsync(query, param).ConfigureAwait(false);
    }

    /// <summary>
    /// InTxAsync はトランザクション内でコールバック fn を実行する（自動リトライ + 自動 COMMIT/ROLLBACK）。
    /// fn が例外を投げた場合は opts.MaxRetries まで自動リトライする（楽観的ロック競合対応）。
    /// </summary>
    // InTxAsync メソッド実装: 自動リトライ付きトランザクションでコールバックを実行する
    public async Task<T> InTxAsync<T>(DistributedTxOptions? opts, Func<IDistributedDbTx, CancellationToken, Task<T>> fn, CancellationToken cancellationToken = default)
    {
        // 最大リトライ回数を設定する（0 = リトライなし）
        var maxRetries = Math.Max(0, opts?.MaxRetries ?? 0);
        // リトライループ
        for (var attempt = 0; attempt <= maxRetries; attempt++)
        {
            // トランザクションを開始する
            await using var tx = await BeginAsync(opts, cancellationToken).ConfigureAwait(false);
            try
            {
                // コールバックを実行する
                var result = await fn(tx, cancellationToken).ConfigureAwait(false);
                // 正常完了の場合はコミットする
                await tx.CommitAsync(cancellationToken).ConfigureAwait(false);
                // 結果を返す
                return result;
            }
            catch (Exception) when (attempt < maxRetries)
            {
                // リトライ可能な場合は継続する（CockroachDB の楽観的ロック競合対応）
                continue;
            }
        }
        // リトライ上限を超えた場合は最後の例外を再スローする（実際にはここに到達しない）
        throw new InvalidOperationException("InTxAsync: リトライ上限を超えた（実装エラー）");
    }

    /// <summary>
    /// PingAsync は DB への接続確認を行う（health check 用途）。
    /// </summary>
    // PingAsync メソッド実装: SELECT 1 を実行して接続確認する
    public async Task PingAsync(CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // SELECT 1 を実行して接続を確認する
        await conn.ExecuteAsync("SELECT 1").ConfigureAwait(false);
    }

    /// <summary>
    /// GetNodeInfoAsync は接続先 Distributed SQL クラスターのノード情報を返す。
    /// CockroachDB の crdb_internal.cluster_settings を参照する。
    /// </summary>
    // GetNodeInfoAsync メソッド実装: クラスターのノード情報を取得する
    public async Task<DistributedDbNodeInfo> GetNodeInfoAsync(CancellationToken cancellationToken = default)
    {
        // 接続を取得する
        await using var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // クラスターバージョンを取得する（CockroachDB の version() 関数）
        var version = await conn.QuerySingleOrDefaultAsync<string>("SELECT version()").ConfigureAwait(false);
        // DistributedDbNodeInfo を構築して返す（簡易実装）
        return new DistributedDbNodeInfo(
            // ノード数: CockroachDB の cluster_settings から取得（簡易実装で 1 を設定する）
            NodeCount: 1,
            // リージョン: 環境変数から取得する（未設定の場合は "local" を返す）
            Region: Environment.GetEnvironmentVariable("CRDB_REGION") ?? "local",
            // バージョン: version() 関数の結果を使用する
            Version: version ?? "unknown",
            // ReadOnly: 常に false（read-write クラスター）
            IsReadOnly: false
        );
    }
}
