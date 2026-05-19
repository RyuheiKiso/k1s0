// DbImpl.cs — k1s0 tier1 Library C# 実装: IDbClient / IDbTx の Npgsql facade 実装
// 13_リレーショナルDB適合仕様.md §IDbClient（PostgreSQL L1+ 深耕）に準拠する。
// Npgsql の NpgsqlConnection を L1+ ラップして公開 API に Npgsql 型を露出しない。
// AuthContext の GUC SET LOCAL を BEGIN 後に自動実行して RLS を強制する。

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
// Npgsql: PostgreSQL クライアント（内部のみ使用する）
using Npgsql;
// Dapper: 軽量 ORM（クエリマッピングに使用する）
using Dapper;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// DbTxImpl は IDbTx の Npgsql NpgsqlTransaction facade 実装クラス。
/// NpgsqlTransaction を内部に隠蔽して公開 API に Npgsql 型を露出しない。
/// IAsyncDisposable を実装して await using で自動 Rollback を実現する。
/// </summary>
// DbTxImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class DbTxImpl : IDbTx
{
    // _conn: Npgsql 接続（internal でのみ参照する）
    private readonly NpgsqlConnection _conn;
    // _tx: Npgsql トランザクション（内部に隠蔽する）
    private readonly NpgsqlTransaction _tx;
    // _disposed: 二重 Dispose 防止フラグ
    private bool _disposed;

    /// <summary>
    /// コンストラクタ: NpgsqlConnection と NpgsqlTransaction を受け取る。
    /// 両方とも internal でのみ参照する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: 接続とトランザクションを受け取る
    internal DbTxImpl(NpgsqlConnection conn, NpgsqlTransaction tx)
    {
        // 接続を保持する
        _conn = conn;
        // トランザクションを保持する
        _tx = tx;
        // 初期状態は未 Dispose
        _disposed = false;
    }

    /// <summary>
    /// QuerySingleOrDefaultAsync は単一行クエリを実行して結果を返す。
    /// Dapper の QuerySingleOrDefaultAsync を使用して型安全にマッピングする。
    /// </summary>
    // QuerySingleOrDefaultAsync メソッド実装: Dapper を通じてクエリを実行する
    public async Task<T?> QuerySingleOrDefaultAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // Dapper の QuerySingleOrDefaultAsync を呼び出す（トランザクション内で実行する）
        return await _conn.QuerySingleOrDefaultAsync<T>(query, param, _tx).ConfigureAwait(false);
    }

    /// <summary>
    /// QueryAsync は複数行クエリを実行して結果リストを返す。
    /// Dapper の QueryAsync を使用して型安全にマッピングする。
    /// </summary>
    // QueryAsync メソッド実装: Dapper を通じてクエリを実行する
    public async Task<IReadOnlyList<T>> QueryAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // Dapper の QueryAsync を呼び出す（トランザクション内で実行する）
        var rows = await _conn.QueryAsync<T>(query, param, _tx).ConfigureAwait(false);
        // IEnumerable を IReadOnlyList に変換して返す
        return rows.ToList().AsReadOnly();
    }

    /// <summary>
    /// ExecuteAsync は DML / DDL を実行して変更された行数を返す。
    /// Dapper の ExecuteAsync を使用して型安全に実行する。
    /// </summary>
    // ExecuteAsync メソッド実装: Dapper を通じて DML を実行する
    public async Task<int> ExecuteAsync(string query, object? param = null, CancellationToken cancellationToken = default)
    {
        // Dapper の ExecuteAsync を呼び出す（トランザクション内で実行する）
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
    /// IAsyncDisposable.DisposeAsync からも呼び出される。
    /// </summary>
    // RollbackAsync メソッド実装: NpgsqlTransaction.RollbackAsync を呼び出す
    public async Task RollbackAsync(CancellationToken cancellationToken = default)
    {
        // Npgsql トランザクションをロールバックする
        await _tx.RollbackAsync(cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// DisposeAsync は await using での自動ロールバックを実現する。
    /// CommitAsync が呼ばれていない場合は自動的にロールバックする。
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
/// DbImpl は IDbClient の Npgsql facade 実装クラス。
/// NpgsqlConnection を L1+ ラップして公開 API に Npgsql 型を露出しない。
/// BEGIN 後に AuthContext.ToGucSetters() を SET LOCAL することで RLS を強制する。
/// </summary>
// DbImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class DbImpl : IDbClient
{
    // _dataSource: Npgsql データソース（接続プール管理に使用する）
    private readonly NpgsqlDataSource _dataSource;
    // _authContext: 認証コンテキスト（GUC SET LOCAL に使用する）
    private readonly AuthContext? _authContext;

    /// <summary>
    /// コンストラクタ: NpgsqlDataSource と AuthContext を注入する。
    /// NpgsqlDataSource は接続プール管理に使用する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: NpgsqlDataSource と AuthContext を依存注入する
    public DbImpl(NpgsqlDataSource dataSource, AuthContext? authContext = null)
    {
        // null チェック: null が渡された場合は例外を投げる
        _dataSource = dataSource ?? throw new ArgumentNullException(nameof(dataSource));
        // 認証コンテキストを保持する（null の場合は GUC SET LOCAL をスキップする）
        _authContext = authContext;
    }

    // ToIsolationLevel は Library の DbTxIsoLevel を System.Data.IsolationLevel に変換する
    private static IsolationLevel ToIsolationLevel(DbTxIsoLevel isoLevel)
    {
        // 各 Library の分離レベルを System.Data.IsolationLevel にマッピングする
        return isoLevel switch
        {
            // ReadCommitted: System.Data.IsolationLevel.ReadCommitted
            DbTxIsoLevel.ReadCommitted => IsolationLevel.ReadCommitted,
            // RepeatableRead: System.Data.IsolationLevel.RepeatableRead
            DbTxIsoLevel.RepeatableRead => IsolationLevel.RepeatableRead,
            // Serializable: System.Data.IsolationLevel.Serializable
            DbTxIsoLevel.Serializable => IsolationLevel.Serializable,
            // 未知の分離レベルは ReadCommitted にフォールバックする
            _ => IsolationLevel.ReadCommitted,
        };
    }

    // ApplyGucSetters は AuthContext の GUC SET LOCAL を接続に適用する
    private async Task ApplyGucSetters(NpgsqlConnection conn, CancellationToken cancellationToken)
    {
        // AuthContext が設定されていない場合はスキップする
        if (_authContext is null) return;
        // ToGucSetters で GUC SET LOCAL 文のリストを取得する
        var setters = _authContext.ToGucSetters();
        // 各 GUC SET LOCAL 文を実行する
        foreach (var setter in setters)
        {
            // SET LOCAL 文を実行する（BeginAsync で呼び出される）
            await using var cmd = conn.CreateCommand();
            // SQL を設定する
            cmd.CommandText = setter;
            // SET LOCAL を実行する
            await cmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);
        }
    }

    /// <summary>
    /// BeginAsync はトランザクションを開始して IDbTx を返す。
    /// BEGIN 後に AuthContext.ToGucSetters() を SET LOCAL して RLS を強制する。
    /// </summary>
    // BeginAsync メソッド実装: トランザクションを開始して GUC を設定する
    public async Task<IDbTx> BeginAsync(DbTxOptions? opts = null, CancellationToken cancellationToken = default)
    {
        // NpgsqlDataSource から新しい接続を取得する
        var conn = await _dataSource.OpenConnectionAsync(cancellationToken).ConfigureAwait(false);
        // 分離レベルを変換する（デフォルト: ReadCommitted）
        var isoLevel = ToIsolationLevel(opts?.IsoLevel ?? DbTxIsoLevel.ReadCommitted);
        // トランザクションを開始する
        var tx = await conn.BeginTransactionAsync(isoLevel, cancellationToken).ConfigureAwait(false);
        // BEGIN 後に GUC SET LOCAL を適用する（RLS 強制）
        await ApplyGucSetters(conn, cancellationToken).ConfigureAwait(false);
        // DbTxImpl を生成して返す
        return new DbTxImpl(conn, tx);
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
    /// InTxAsync はトランザクション内でコールバック fn を実行する（COMMIT / ROLLBACK は自動管理）。
    /// fn が例外を投げた場合は自動 ROLLBACK する。
    /// </summary>
    // InTxAsync メソッド実装: トランザクション内でコールバックを実行する
    public async Task<T> InTxAsync<T>(DbTxOptions? opts, Func<IDbTx, CancellationToken, Task<T>> fn, CancellationToken cancellationToken = default)
    {
        // トランザクションを開始する
        await using var tx = await BeginAsync(opts, cancellationToken).ConfigureAwait(false);
        // コールバックを実行する（例外が発生した場合は DisposeAsync で自動ロールバック）
        var result = await fn(tx, cancellationToken).ConfigureAwait(false);
        // 正常完了の場合はコミットする
        await tx.CommitAsync(cancellationToken).ConfigureAwait(false);
        // 結果を返す
        return result;
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
    /// GetPoolStats は接続プールの統計情報を返す（監視 / メトリクス用途）。
    /// Npgsql 8.0.x では Statistics プロパティが内部型のため、デフォルト値を返す。
    /// 詳細な統計情報が必要な場合は NpgsqlDataSource を直接使用すること。
    /// </summary>
    // GetPoolStats メソッド実装: 接続プールの統計情報を返す（Npgsql 8.0.x 互換）
    public DbPoolStats GetPoolStats()
    {
        // Npgsql 8.0.x では NpgsqlDataSource.Statistics は protected/internal のため直接アクセスできない
        // デフォルト値を返して互換性を保つ（実際の統計は外部監視（Prometheus / OTEL）経由で取得する）
        return new DbPoolStats(
            // 総接続数: 不明のため 0 を返す
            TotalConnections: 0,
            // アイドル接続数: 不明のため 0 を返す
            IdleConnections: 0,
            // 使用中接続数: 不明のため 0 を返す
            AcquiredConnections: 0,
            // 接続待ちリクエスト数: 不明のため 0 を返す
            WaitCount: 0,
            // 最大接続数: 不明のため 0 を返す
            MaxConnections: 0
        );
    }
}
