// IWorkflow.cs — k1s0 tier1 Library C# 実装: Workflow / Long-running Saga の L1+ interface
// 16_ワークフロー適合仕様.md §IWorkflowClient（Temporal L1+ 深耕）に準拠する。
// Temporal の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
// OSS 型（Temporalio.Client 等）を公開シグネチャに一切含まない。

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
/// WorkflowStatus はワークフロー実行の状態を宣言する enum。
/// Temporal の workflow execution status に準拠した Library 独自語彙とする。
/// </summary>
// WorkflowStatus 列挙型定義
public enum WorkflowStatus
{
    /// <summary>Running: 実行中</summary>
    Running,
    /// <summary>Completed: 正常完了</summary>
    Completed,
    /// <summary>Failed: 失敗（リトライ上限超過）</summary>
    Failed,
    /// <summary>Canceled: キャンセルされた</summary>
    Canceled,
    /// <summary>TimedOut: タイムアウト</summary>
    TimedOut,
    /// <summary>ContinuedAsNew: 継続（ContinueAsNew パターン）</summary>
    ContinuedAsNew,
    /// <summary>Terminated: 強制終了</summary>
    Terminated,
}

/// <summary>
/// WorkflowOptions はワークフロー開始オプションを宣言する型。
/// Temporal の StartWorkflowOptions を Library 独自語彙に翻訳する。
/// </summary>
// WorkflowOptions クラス定義
public sealed class WorkflowOptions
{
    /// <summary>WorkflowId: ワークフロー識別子（idempotency key として使用する）</summary>
    // WorkflowId プロパティ（必須）
    public required string WorkflowId { get; init; }

    /// <summary>TaskQueue: 実行するタスクキュー名</summary>
    // TaskQueue プロパティ（必須）
    public required string TaskQueue { get; init; }

    /// <summary>MaxRetries: ワークフロー全体のリトライ上限回数（0 = リトライなし）</summary>
    // MaxRetries プロパティ
    public int MaxRetries { get; init; }

    /// <summary>RetentionDays: ワークフロー履歴の保持日数（0 = デフォルト設定を使用する）</summary>
    // RetentionDays プロパティ
    public int RetentionDays { get; init; }

    /// <summary>SearchAttributes: Temporal Search Attribute（ダッシュボード検索用）</summary>
    // SearchAttributes プロパティ
    public IReadOnlyDictionary<string, object?>? SearchAttributes { get; init; }

    /// <summary>Memo: ワークフローメモ（非インデックスの補足情報）</summary>
    // Memo プロパティ
    public IReadOnlyDictionary<string, object?>? Memo { get; init; }
}

/// <summary>
/// WorkflowExecution はワークフロー実行の識別子を宣言する型。
/// </summary>
// WorkflowExecution レコード定義
public sealed record WorkflowExecution(
    string WorkflowId,
    string RunId,
    string TenantId
);

/// <summary>
/// IWorkflowRun はワークフロー実行ハンドルを宣言する interface。
/// Temporal の WorkflowHandle を Library 独自語彙で表現する。
/// </summary>
// IWorkflowRun インターフェース定義
public interface IWorkflowRun
{
    /// <summary>WorkflowId はワークフロー識別子を返す。</summary>
    // WorkflowId プロパティ
    string WorkflowId { get; }

    /// <summary>RunId は実行 ID を返す。</summary>
    // RunId プロパティ
    string RunId { get; }

    /// <summary>
    /// GetResultAsync はワークフローの完了を待機して結果を取得する。
    /// </summary>
    // GetResultAsync メソッド: ワークフロー完了を待機して結果を取得する
    Task<T> GetResultAsync<T>(CancellationToken cancellationToken = default);

    /// <summary>
    /// GetStatusAsync はワークフローの現在の状態を返す。
    /// </summary>
    // GetStatusAsync メソッド: 現在の状態を返す
    Task<WorkflowStatus> GetStatusAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// WorkflowDescription はワークフロー実行の詳細情報を宣言する型。
/// </summary>
// WorkflowDescription クラス定義
public sealed class WorkflowDescription
{
    /// <summary>Execution: ワークフロー実行の識別子</summary>
    // Execution プロパティ（必須）
    public required WorkflowExecution Execution { get; init; }

    /// <summary>Status: 現在の状態</summary>
    // Status プロパティ
    public WorkflowStatus Status { get; init; }

    /// <summary>WorkflowType: ワークフロータイプ名</summary>
    // WorkflowType プロパティ（必須）
    public required string WorkflowType { get; init; }

    /// <summary>StartTimeMs: 開始時刻（Unix ミリ秒）</summary>
    // StartTimeMs プロパティ
    public long StartTimeMs { get; init; }

    /// <summary>CloseTimeMs: 終了時刻（Unix ミリ秒: 実行中の場合は 0）</summary>
    // CloseTimeMs プロパティ
    public long CloseTimeMs { get; init; }

    /// <summary>SearchAttributes: 検索属性</summary>
    // SearchAttributes プロパティ
    public IReadOnlyDictionary<string, object?>? SearchAttributes { get; init; }

    /// <summary>Memo: ワークフローメモ</summary>
    // Memo プロパティ
    public IReadOnlyDictionary<string, object?>? Memo { get; init; }
}

/// <summary>
/// IWorkflowClient は Workflow / Long-running Saga の L1+ 抽象 interface を宣言する。
/// Temporal の full API を Library 独自語彙で表現する。
/// OSS 型（Temporalio.Client.ITemporalClient 等）を一切含まない。
/// TenantId を必須として AuthContext 伝播を強制する（tenant 分離必須）。
/// </summary>
// IWorkflowClient インターフェース定義
public interface IWorkflowClient
{
    /// <summary>
    /// StartWorkflowAsync はワークフローを開始して IWorkflowRun を返す。
    /// workflowType はワークフロー登録名。
    /// args はワークフロー開始引数。
    /// </summary>
    // StartWorkflowAsync メソッド: ワークフローを開始する
    Task<IWorkflowRun> StartWorkflowAsync(
        string workflowType,
        WorkflowOptions opts,
        object?[]? args = null,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// SignalWorkflowAsync は実行中のワークフローにシグナルを送信する。
    /// runId = "" で最新の実行に送信する。
    /// </summary>
    // SignalWorkflowAsync メソッド: シグナルを送信する
    Task SignalWorkflowAsync(
        string tenantId,
        string workflowId,
        string runId,
        string signalName,
        object? arg = null,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// QueryWorkflowAsync は実行中のワークフローにクエリを送信して結果を取得する。
    /// </summary>
    // QueryWorkflowAsync メソッド: クエリを送信する
    Task<T> QueryWorkflowAsync<T>(
        string tenantId,
        string workflowId,
        string runId,
        string queryType,
        object?[]? args = null,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// CancelWorkflowAsync は実行中のワークフローをキャンセルする（グレースフルキャンセル）。
    /// </summary>
    // CancelWorkflowAsync メソッド: ワークフローをキャンセルする
    Task CancelWorkflowAsync(string tenantId, string workflowId, string runId, CancellationToken cancellationToken = default);

    /// <summary>
    /// TerminateWorkflowAsync は実行中のワークフローを強制終了する（理由を指定する）。
    /// </summary>
    // TerminateWorkflowAsync メソッド: ワークフローを強制終了する
    Task TerminateWorkflowAsync(string tenantId, string workflowId, string runId, string reason, CancellationToken cancellationToken = default);

    /// <summary>
    /// DescribeWorkflowAsync はワークフローの詳細情報を取得する。
    /// </summary>
    // DescribeWorkflowAsync メソッド: ワークフローの詳細情報を取得する
    Task<WorkflowDescription> DescribeWorkflowAsync(string tenantId, string workflowId, string runId, CancellationToken cancellationToken = default);
}
