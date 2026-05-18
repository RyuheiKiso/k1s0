// IRelationalStore.cs — k1s0 tier1 Library C# 実装: Relational Store / Single-leader の L1+ interface
// 13_リレーショナルDB適合仕様.md §IDbClient（PostgreSQL L1+ 深耕）に準拠する。
// PostgreSQL の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と RLS を強制する。
// OSS 型（Npgsql 等）を公開シグネチャに一切含まない。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyList に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// DbTxIsoLevel はトランザクション分離レベルを宣言する enum。
/// PostgreSQL の isolation level に準拠した Library 独自語彙とする。
/// </summary>
// DbTxIsoLevel 列挙型定義
public enum DbTxIsoLevel
{
    /// <summary>ReadCommitted: Read Committed（デフォルト: RLS と組み合わせて tenant 分離を保証する）</summary>
    ReadCommitted,
    /// <summary>RepeatableRead: Repeatable Read（整合性スナップショット読み取り）</summary>
    RepeatableRead,
    /// <summary>Serializable: Serializable（SSI: 完全な直列化保証）</summary>
    Serializable,
}

/// <summary>
/// DbTxOptions はトランザクション開始オプションを宣言する型。
/// </summary>
// DbTxOptions クラス定義
public sealed class DbTxOptions
{
    /// <summary>IsoLevel: トランザクション分離レベル（デフォルト: DbTxIsoLevel.ReadCommitted）</summary>
    // IsoLevel プロパティ
    public DbTxIsoLevel IsoLevel { get; init; } = DbTxIsoLevel.ReadCommitted;

    /// <summary>ReadOnly: 読み取り専用トランザクションかどうか（true = BEGIN READ ONLY）</summary>
    // ReadOnly プロパティ
    public bool ReadOnly { get; init; }

    /// <summary>DeferConstraints: 制約チェックを DEFERRED にするかどうか</summary>
    // DeferConstraints プロパティ
    public bool DeferConstraints { get; init; }
}

/// <summary>
/// IDbTx はアクティブなトランザクションを宣言する interface。
/// OSS の NpgsqlTransaction 等を露出せず Library 独自語彙で表現する。
/// IAsyncDisposable を実装して await using で自動 Rollback を実現する。
/// </summary>
// IDbTx インターフェース定義
public interface IDbTx : IAsyncDisposable
{
    /// <summary>
    /// QuerySingleOrDefaultAsync は単一行クエリを実行して結果を返す（行が存在しない場合は default(T)）。
    /// query はプリペアドクエリの SQL テンプレート（生 SQL 文字列組み立て禁止）。
    /// </summary>
    // QuerySingleOrDefaultAsync メソッド: 単一行クエリを実行する
    Task<T?> QuerySingleOrDefaultAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// QueryAsync は複数行クエリを実行して結果リストを返す。
    /// </summary>
    // QueryAsync メソッド: 複数行クエリを実行する
    Task<IReadOnlyList<T>> QueryAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// ExecuteAsync は DML / DDL を実行して変更された行数を返す。
    /// </summary>
    // ExecuteAsync メソッド: DML を実行する
    Task<int> ExecuteAsync(string query, object? param = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// CommitAsync はトランザクションをコミットする。
    /// </summary>
    // CommitAsync メソッド: トランザクションをコミットする
    Task CommitAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// RollbackAsync はトランザクションをロールバックする（IAsyncDisposable.DisposeAsync でも自動呼び出しされる）。
    /// </summary>
    // RollbackAsync メソッド: トランザクションをロールバックする
    Task RollbackAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// IDbClient は Relational Store / Single-leader の L1+ 抽象 interface を宣言する。
/// PostgreSQL の full API を Library 独自語彙で表現する。
/// OSS 型（Npgsql 等）を引数・戻り値に一切含まない。
/// AuthContext が伝播されていることを前提とする（RLS / GUC 設定のため必須）。
/// </summary>
// IDbClient インターフェース定義
public interface IDbClient
{
    /// <summary>
    /// BeginAsync はトランザクションを開始して IDbTx を返す。
    /// 実装側は BEGIN 後に AuthContext.GetGucSetters() で GUC を SET LOCAL する。
    /// </summary>
    // BeginAsync メソッド: トランザクションを開始する
    Task<IDbTx> BeginAsync(DbTxOptions? opts = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// QuerySingleOrDefaultAsync は単一行クエリを autocommit モードで実行する。
    /// </summary>
    // QuerySingleOrDefaultAsync メソッド: 単一行クエリを autocommit で実行する
    Task<T?> QuerySingleOrDefaultAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// QueryAsync は複数行クエリを autocommit モードで実行する。
    /// </summary>
    // QueryAsync メソッド: 複数行クエリを autocommit で実行する
    Task<IReadOnlyList<T>> QueryAsync<T>(string query, object? param = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// ExecuteAsync は DML を autocommit モードで実行する。
    /// </summary>
    // ExecuteAsync メソッド: DML を autocommit で実行する
    Task<int> ExecuteAsync(string query, object? param = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// InTxAsync はトランザクション内でコールバック fn を実行する（COMMIT / ROLLBACK は自動管理）。
    /// fn が例外を投げた場合は自動 ROLLBACK する。
    /// fn が正常完了した場合は自動 COMMIT する。
    /// </summary>
    // InTxAsync メソッド: トランザクション内でコールバックを実行する
    Task<T> InTxAsync<T>(DbTxOptions? opts, Func<IDbTx, CancellationToken, Task<T>> fn, CancellationToken cancellationToken = default);

    /// <summary>
    /// PingAsync は DB への接続確認を行う（health check 用途）。
    /// </summary>
    // PingAsync メソッド: 接続確認を行う
    Task PingAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// GetPoolStats は接続プールの統計情報を返す（監視 / メトリクス用途）。
    /// </summary>
    // GetPoolStats メソッド: 接続プールの統計情報を返す
    DbPoolStats GetPoolStats();
}

/// <summary>
/// DbPoolStats は接続プールの統計情報を宣言する型。
/// OSS の NpgsqlConnectionPool stats を露出せず Library 独自語彙で表現する。
/// </summary>
// DbPoolStats レコード定義
public sealed record DbPoolStats(
    // TotalConnections: 接続プールの総接続数
    int TotalConnections,
    // IdleConnections: アイドル状態の接続数
    int IdleConnections,
    // AcquiredConnections: 使用中の接続数
    int AcquiredConnections,
    // WaitCount: 接続待ちリクエスト数
    long WaitCount,
    // MaxConnections: 接続プールの最大接続数設定値
    int MaxConnections
);
