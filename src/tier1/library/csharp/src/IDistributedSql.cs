// IDistributedSql.cs — k1s0 tier1 Library C# 実装: Relational Store / Distributed SQL の L1+ interface
// 14_分散SQL適合仕様.md §IDistributedDbClient（CockroachDB / Spanner L1+ 深耕）に準拠する。
// Distributed SQL の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
// OSS 型（Npgsql / Google.Cloud.Spanner 等）を公開シグネチャに一切含まない。

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
/// DistributedTxPriority はトランザクションの実行優先度を宣言する enum。
/// CockroachDB の transaction priority に準拠した Library 独自語彙とする。
/// </summary>
// DistributedTxPriority 列挙型定義
public enum DistributedTxPriority
{
    /// <summary>Normal: 通常優先度（デフォルト）</summary>
    Normal,
    /// <summary>High: 高優先度（コンテンション時に優先してコミットする）</summary>
    High,
    /// <summary>Low: 低優先度（バックグラウンドジョブに使用する）</summary>
    Low,
}

/// <summary>
/// DistributedTxOptions は Distributed SQL のトランザクション開始オプションを宣言する型。
/// DbTxOptions を継承しつつ、Distributed SQL 固有オプションを追加する。
/// </summary>
// DistributedTxOptions クラス定義
public sealed class DistributedTxOptions
{
    /// <summary>IsoLevel: トランザクション分離レベル（Distributed SQL では Serializable 推奨）</summary>
    // IsoLevel プロパティ
    public DbTxIsoLevel IsoLevel { get; init; } = DbTxIsoLevel.Serializable;

    /// <summary>ReadOnly: 読み取り専用トランザクションかどうか</summary>
    // ReadOnly プロパティ
    public bool ReadOnly { get; init; }

    /// <summary>Priority: Distributed SQL のトランザクション優先度</summary>
    // Priority プロパティ
    public DistributedTxPriority Priority { get; init; } = DistributedTxPriority.Normal;

    /// <summary>
    /// AsOfSystemTimeTick: 過去の特定タイムスタンプを読む（AOST: CockroachDB 固有機能）。
    /// wall-clock TTL 禁止規約に準拠して HLC tick 値で指定する（0 = 最新を読む）。
    /// </summary>
    // AsOfSystemTimeTick プロパティ
    public ulong AsOfSystemTimeTick { get; init; }

    /// <summary>MaxRetries: 自動リトライ回数（楽観的ロック競合時に自動リトライする）</summary>
    // MaxRetries プロパティ
    public int MaxRetries { get; init; }
}

/// <summary>
/// IDistributedDbTx は Distributed SQL のアクティブなトランザクションを宣言する interface。
/// IDbTx を継承しつつ、Distributed SQL 固有メソッドを追加する。
/// </summary>
// IDistributedDbTx インターフェース定義
public interface IDistributedDbTx : IDbTx
{
    /// <summary>
    /// SavepointAsync は名前付きセーブポイントを作成する（ネストトランザクション用）。
    /// name はセーブポイント名（英数字 + アンダースコアのみ許容する）。
    /// </summary>
    // SavepointAsync メソッド: セーブポイントを作成する
    Task SavepointAsync(string name, CancellationToken cancellationToken = default);

    /// <summary>
    /// RollbackToSavepointAsync はセーブポイントにロールバックする（部分ロールバック）。
    /// </summary>
    // RollbackToSavepointAsync メソッド: セーブポイントにロールバックする
    Task RollbackToSavepointAsync(string name, CancellationToken cancellationToken = default);

    /// <summary>
    /// ReleaseSavepointAsync はセーブポイントをリリースする（セーブポイントを確定する）。
    /// </summary>
    // ReleaseSavepointAsync メソッド: セーブポイントをリリースする
    Task ReleaseSavepointAsync(string name, CancellationToken cancellationToken = default);

    /// <summary>Priority は現在のトランザクション優先度を返す。</summary>
    // Priority プロパティ
    DistributedTxPriority Priority { get; }
}

/// <summary>
/// IDistributedDbClient は Relational Store / Distributed SQL の L1+ 抽象 interface を宣言する。
/// CockroachDB / Google Cloud Spanner / YugabyteDB 等を抽象化する。
/// OSS 型を引数・戻り値に一切含まない。
/// 単一ノード PostgreSQL と区別するために IDbClient を継承しない（別 interface として宣言する）。
/// </summary>
// IDistributedDbClient インターフェース定義
public interface IDistributedDbClient
{
    /// <summary>
    /// BeginAsync はトランザクションを開始して IDistributedDbTx を返す。
    /// opts は Distributed SQL 固有オプション（priority / AOST 等）。
    /// </summary>
    // BeginAsync メソッド: トランザクションを開始する
    Task<IDistributedDbTx> BeginAsync(DistributedTxOptions? opts = null, CancellationToken cancellationToken = default);

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
    /// InTxAsync はトランザクション内でコールバック fn を実行する（自動リトライ + 自動 COMMIT/ROLLBACK）。
    /// fn が例外を投げた場合は自動 ROLLBACK 後に opts.MaxRetries まで自動リトライする。
    /// </summary>
    // InTxAsync メソッド: トランザクション内でコールバックを実行する
    Task<T> InTxAsync<T>(DistributedTxOptions? opts, Func<IDistributedDbTx, CancellationToken, Task<T>> fn, CancellationToken cancellationToken = default);

    /// <summary>
    /// PingAsync は DB への接続確認を行う（health check 用途）。
    /// </summary>
    // PingAsync メソッド: 接続確認を行う
    Task PingAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// GetNodeInfoAsync は接続先 Distributed SQL クラスターのノード情報を返す（診断用）。
    /// </summary>
    // GetNodeInfoAsync メソッド: クラスターのノード情報を取得する
    Task<DistributedDbNodeInfo> GetNodeInfoAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// DistributedDbNodeInfo は Distributed SQL クラスターのノード情報を宣言する型。
/// </summary>
// DistributedDbNodeInfo レコード定義
public sealed record DistributedDbNodeInfo(
    int NodeCount,
    string Region,
    string Version,
    bool IsReadOnly
);
