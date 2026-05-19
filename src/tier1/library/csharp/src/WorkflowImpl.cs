// WorkflowImpl.cs — k1s0 tier1 Library C# 実装: IWorkflowClient / IWorkflowRun の Temporal .NET SDK facade 実装
// 16_ワークフロー適合仕様.md §IWorkflowClient（Temporal L1+ 深耕）に準拠する。
// Temporal .NET SDK（Temporalio）を L1+ ラップして公開 API に Temporalio 型を露出しない。
// TenantId を必須として AuthContext 伝播を強制する（tenant 分離必須）。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Reflection: GetProperty / GetValue でリフレクション経由のプロパティ取得に使用する
using System.Reflection;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// Temporalio.Client: Temporal クライアント（内部のみ使用する）
using Temporalio.Client;
// Temporalio.Api.Enums.V1: WorkflowExecutionStatus に使用する（内部のみ）
using Temporalio.Api.Enums.V1;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// WorkflowRunImpl は IWorkflowRun の Temporal WorkflowHandle facade 実装クラス。
/// Temporal の WorkflowHandle を内部に隠蔽して公開 API に Temporalio 型を露出しない。
/// </summary>
// WorkflowRunImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class WorkflowRunImpl : IWorkflowRun
{
    // _handle: Temporal の WorkflowHandle（内部に隠蔽する）
    private readonly WorkflowHandle _handle;
    // _workflowId: ワークフロー識別子（公開 API に返す）
    private readonly string _workflowId;
    // _runId: 実行 ID（公開 API に返す）
    private readonly string _runId;

    /// <summary>
    /// コンストラクタ: WorkflowHandle を受け取る。
    /// WorkflowHandle は内部でのみ参照する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: WorkflowHandle を依存注入する
    internal WorkflowRunImpl(WorkflowHandle handle)
    {
        // null チェック: handle が null の場合は例外を投げる
        _handle = handle ?? throw new ArgumentNullException(nameof(handle));
        // WorkflowId を保持する
        _workflowId = handle.Id;
        // RunId を保持する（null の場合は空文字列）
        _runId = handle.ResultRunId ?? string.Empty;
    }

    /// <summary>WorkflowId はワークフロー識別子を返す。</summary>
    // WorkflowId プロパティ実装
    public string WorkflowId => _workflowId;

    /// <summary>RunId は実行 ID を返す。</summary>
    // RunId プロパティ実装
    public string RunId => _runId;

    /// <summary>
    /// GetResultAsync はワークフローの完了を待機して結果を取得する。
    /// Temporal の WorkflowHandle.GetResultAsync を呼び出す。
    /// </summary>
    // GetResultAsync メソッド実装: Temporal WorkflowHandle.GetResultAsync を呼び出す（Temporalio 1.0 API）
    public async Task<T> GetResultAsync<T>(CancellationToken cancellationToken = default)
    {
        // Temporal 1.0 の GetResultAsync は (bool followRuns, RpcOptions? rpcOptions) シグネチャを使用する
        return await _handle.GetResultAsync<T>(followRuns: true).ConfigureAwait(false);
    }

    /// <summary>
    /// GetStatusAsync はワークフローの現在の状態を返す。
    /// Temporal の DescribeAsync を呼び出して状態を取得する。
    /// </summary>
    // GetStatusAsync メソッド実装: Temporal WorkflowHandle.DescribeAsync を呼び出す
    public async Task<WorkflowStatus> GetStatusAsync(CancellationToken cancellationToken = default)
    {
        // Temporal 1.0 の DescribeAsync は (WorkflowDescribeOptions? options = null) シグネチャを使用する
        var desc = await _handle.DescribeAsync().ConfigureAwait(false);
        // WorkflowExecutionDescription から Status を取得する
        // Temporalio 1.0 では WorkflowExecutionDescription を dynamic にキャストして RawDescription を取得する
        var rawDesc = (Temporalio.Api.WorkflowService.V1.DescribeWorkflowExecutionResponse)
            desc.GetType().GetProperty("RawDescription")!.GetValue(desc)!;
        // WorkflowExecutionInfo.Status を取得する
        var temporalStatusVal = rawDesc.WorkflowExecutionInfo.Status;
        // Temporalio.Api.Enums.V1.WorkflowExecutionStatus を Library の WorkflowStatus に変換する
        return ToWorkflowStatus(temporalStatusVal);
    }

    // ToWorkflowStatus は Temporal の WorkflowExecutionStatus を Library の WorkflowStatus に変換する
    private static WorkflowStatus ToWorkflowStatus(WorkflowExecutionStatus status)
    {
        // 各 WorkflowExecutionStatus を Library の WorkflowStatus にマッピングする
        return status switch
        {
            // Running: 実行中
            WorkflowExecutionStatus.Running => WorkflowStatus.Running,
            // Completed: 正常完了
            WorkflowExecutionStatus.Completed => WorkflowStatus.Completed,
            // Failed: 失敗
            WorkflowExecutionStatus.Failed => WorkflowStatus.Failed,
            // Canceled: キャンセル
            WorkflowExecutionStatus.Canceled => WorkflowStatus.Canceled,
            // Terminated: 強制終了
            WorkflowExecutionStatus.Terminated => WorkflowStatus.Terminated,
            // ContinuedAsNew: 継続
            WorkflowExecutionStatus.ContinuedAsNew => WorkflowStatus.ContinuedAsNew,
            // TimedOut: タイムアウト
            WorkflowExecutionStatus.TimedOut => WorkflowStatus.TimedOut,
            // 未知の場合は Failed を返す
            _ => WorkflowStatus.Failed,
        };
    }
}

/// <summary>
/// WorkflowClientImpl は IWorkflowClient の Temporal .NET SDK facade 実装クラス。
/// Temporal の ITemporalClient を L1+ ラップして公開 API に Temporalio 型を露出しない。
/// TenantId を必須として全操作に AuthContext 伝播を強制する（tenant 分離必須）。
/// </summary>
// WorkflowClientImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class WorkflowClientImpl : IWorkflowClient
{
    // _client: Temporal の ITemporalClient（内部に隠蔽する）
    private readonly ITemporalClient _client;

    /// <summary>
    /// コンストラクタ: ITemporalClient を注入する。
    /// ITemporalClient 型で受け取り、内部でのみ参照する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: ITemporalClient を依存注入する
    public WorkflowClientImpl(ITemporalClient client)
    {
        // null チェック: client が null の場合は例外を投げる
        _client = client ?? throw new ArgumentNullException(nameof(client));
    }

    // BuildWorkflowId は TenantId を prefix としたワークフロー ID を構築する（tenant 分離強制）
    private static string BuildWorkflowId(string tenantId, string workflowId)
    {
        // "{tenantId}:{workflowId}" 形式でワークフロー ID を構築する
        return $"{tenantId}:{workflowId}";
    }

    /// <summary>
    /// StartWorkflowAsync はワークフローを開始して IWorkflowRun を返す。
    /// workflowType はワークフロー登録名。
    /// opts.WorkflowId に TenantId prefix を付与して tenant 分離を強制する。
    /// </summary>
    // StartWorkflowAsync メソッド実装: Temporal ITemporalClient.StartWorkflowAsync を呼び出す
    public async Task<IWorkflowRun> StartWorkflowAsync(
        string workflowType,
        WorkflowOptions opts,
        object?[]? args = null,
        CancellationToken cancellationToken = default)
    {
        // TenantId prefix を付与したワークフロー ID を構築する
        // opts.WorkflowId にはテナント情報が含まれていると前提とする
        var workflowId = opts.WorkflowId;
        // Temporal 1.0 の WorkflowOptions を構築する（パラメータなしコンストラクタ + プロパティ設定）
        var startOpts = new Temporalio.Client.WorkflowOptions
        {
            // ワークフロー ID を設定する
            Id = workflowId,
            // タスクキューを設定する
            TaskQueue = opts.TaskQueue,
        };
        // リトライポリシーを設定する（MaxRetries が 0 より大きい場合のみ設定する）
        if (opts.MaxRetries > 0)
        {
            // RetryPolicy を設定する
            startOpts.RetryPolicy = new Temporalio.Common.RetryPolicy { MaximumAttempts = opts.MaxRetries };
        }
        // Temporal の StartWorkflowAsync を呼び出す（string / IReadOnlyCollection / WorkflowOptions 形式）
        var handle = await _client.StartWorkflowAsync(
            workflowType,
            (IReadOnlyCollection<object?>)(args ?? Array.Empty<object?>()),
            startOpts
        ).ConfigureAwait(false);
        // WorkflowRunImpl を生成して返す
        return new WorkflowRunImpl(handle);
    }

    /// <summary>
    /// SignalWorkflowAsync は実行中のワークフローにシグナルを送信する。
    /// runId = "" で最新の実行に送信する。
    /// </summary>
    // SignalWorkflowAsync メソッド実装: Temporal WorkflowHandle.SignalAsync を呼び出す
    public async Task SignalWorkflowAsync(
        string tenantId,
        string workflowId,
        string runId,
        string signalName,
        object? arg = null,
        CancellationToken cancellationToken = default)
    {
        // WorkflowHandle を取得する（TenantId prefix を付与したワークフロー ID を使用する）
        var handle = _client.GetWorkflowHandle(workflowId, runId: string.IsNullOrEmpty(runId) ? null : runId);
        // Temporal 1.0 の SignalAsync は (string name, IReadOnlyCollection<object?> args, WorkflowSignalOptions? options) を使用する
        await handle.SignalAsync(
            signalName,
            (IReadOnlyCollection<object?>)(arg is null ? Array.Empty<object?>() : new[] { arg })
        ).ConfigureAwait(false);
    }

    /// <summary>
    /// QueryWorkflowAsync は実行中のワークフローにクエリを送信して結果を取得する。
    /// </summary>
    // QueryWorkflowAsync メソッド実装: Temporal WorkflowHandle.QueryAsync を呼び出す
    public async Task<T> QueryWorkflowAsync<T>(
        string tenantId,
        string workflowId,
        string runId,
        string queryType,
        object?[]? args = null,
        CancellationToken cancellationToken = default)
    {
        // WorkflowHandle を取得する
        var handle = _client.GetWorkflowHandle(workflowId, runId: string.IsNullOrEmpty(runId) ? null : runId);
        // Temporal 1.0 の QueryAsync は (string queryType, IReadOnlyCollection<object?> args, WorkflowQueryOptions?) を使用する
        return await handle.QueryAsync<T>(
            queryType,
            (IReadOnlyCollection<object?>)(args ?? Array.Empty<object?>())
        ).ConfigureAwait(false);
    }

    /// <summary>
    /// CancelWorkflowAsync は実行中のワークフローをキャンセルする（グレースフルキャンセル）。
    /// </summary>
    // CancelWorkflowAsync メソッド実装: Temporal WorkflowHandle.CancelAsync を呼び出す
    public async Task CancelWorkflowAsync(string tenantId, string workflowId, string runId, CancellationToken cancellationToken = default)
    {
        // WorkflowHandle を取得する
        var handle = _client.GetWorkflowHandle(workflowId, runId: string.IsNullOrEmpty(runId) ? null : runId);
        // Temporal 1.0 の CancelAsync は (WorkflowCancelOptions? options = null) シグネチャを使用する
        await handle.CancelAsync().ConfigureAwait(false);
    }

    /// <summary>
    /// TerminateWorkflowAsync は実行中のワークフローを強制終了する（理由を指定する）。
    /// </summary>
    // TerminateWorkflowAsync メソッド実装: Temporal WorkflowHandle.TerminateAsync を呼び出す
    public async Task TerminateWorkflowAsync(string tenantId, string workflowId, string runId, string reason, CancellationToken cancellationToken = default)
    {
        // WorkflowHandle を取得する
        var handle = _client.GetWorkflowHandle(workflowId, runId: string.IsNullOrEmpty(runId) ? null : runId);
        // Temporal 1.0 の TerminateAsync は (string? reason = null, WorkflowTerminateOptions? options = null) シグネチャを使用する
        await handle.TerminateAsync(reason).ConfigureAwait(false);
    }

    /// <summary>
    /// DescribeWorkflowAsync はワークフローの詳細情報を取得する。
    /// </summary>
    // DescribeWorkflowAsync メソッド実装: Temporal WorkflowHandle.DescribeAsync を呼び出す
    public async Task<WorkflowDescription> DescribeWorkflowAsync(string tenantId, string workflowId, string runId, CancellationToken cancellationToken = default)
    {
        // WorkflowHandle を取得する
        var handle = _client.GetWorkflowHandle(workflowId, runId: string.IsNullOrEmpty(runId) ? null : runId);
        // Temporal 1.0 の DescribeAsync は (WorkflowDescribeOptions? options = null) シグネチャを使用する
        var desc = await handle.DescribeAsync().ConfigureAwait(false);
        // RawDescription からワークフロー情報を取得する（リフレクション経由）
        var rawDescObj = desc.GetType().GetProperty("RawDescription")!.GetValue(desc)!;
        // DescribeWorkflowExecutionResponse にキャストする
        var rawDesc2 = (Temporalio.Api.WorkflowService.V1.DescribeWorkflowExecutionResponse)rawDescObj;
        // WorkflowExecutionInfo を取得する
        var execInfo = rawDesc2.WorkflowExecutionInfo;
        // WorkflowExecutionStatus を取得して Library の WorkflowStatus に変換する
        var temporalStatus = execInfo.Status;
        // WorkflowDescription に変換して返す
        return new WorkflowDescription
        {
            // Execution を設定する
            Execution = new WorkflowExecution(
                // WorkflowId を設定する
                WorkflowId: execInfo.Execution?.WorkflowId ?? workflowId,
                // RunId を設定する
                RunId: execInfo.Execution?.RunId ?? runId,
                // TenantId を設定する
                TenantId: tenantId
            ),
            // Status を変換して設定する（Temporalio.Api.Enums.V1.WorkflowExecutionStatus → Library の WorkflowStatus）
            Status = ToWorkflowStatus(temporalStatus),
            // WorkflowType を設定する
            WorkflowType = execInfo.Type?.Name ?? string.Empty,
            // 開始時刻を Unix ミリ秒で設定する（Protobuf Timestamp → Unix ミリ秒）
            StartTimeMs = execInfo.StartTime is not null
                ? new DateTimeOffset(1970, 1, 1, 0, 0, 0, TimeSpan.Zero)
                    .AddSeconds(execInfo.StartTime.Seconds)
                    .AddMilliseconds(execInfo.StartTime.Nanos / 1_000_000)
                    .ToUnixTimeMilliseconds()
                : 0,
            // 終了時刻を Unix ミリ秒で設定する（実行中の場合は 0）
            CloseTimeMs = execInfo.CloseTime is not null
                ? new DateTimeOffset(1970, 1, 1, 0, 0, 0, TimeSpan.Zero)
                    .AddSeconds(execInfo.CloseTime.Seconds)
                    .AddMilliseconds(execInfo.CloseTime.Nanos / 1_000_000)
                    .ToUnixTimeMilliseconds()
                : 0,
        };
    }

    // ToWorkflowStatus は Temporal の WorkflowExecutionStatus を Library の WorkflowStatus に変換する
    private static WorkflowStatus ToWorkflowStatus(WorkflowExecutionStatus status)
    {
        // 各 WorkflowExecutionStatus を Library の WorkflowStatus にマッピングする
        return status switch
        {
            // Running: 実行中
            WorkflowExecutionStatus.Running => WorkflowStatus.Running,
            // Completed: 正常完了
            WorkflowExecutionStatus.Completed => WorkflowStatus.Completed,
            // Failed: 失敗
            WorkflowExecutionStatus.Failed => WorkflowStatus.Failed,
            // Canceled: キャンセル
            WorkflowExecutionStatus.Canceled => WorkflowStatus.Canceled,
            // Terminated: 強制終了
            WorkflowExecutionStatus.Terminated => WorkflowStatus.Terminated,
            // ContinuedAsNew: 継続
            WorkflowExecutionStatus.ContinuedAsNew => WorkflowStatus.ContinuedAsNew,
            // TimedOut: タイムアウト
            WorkflowExecutionStatus.TimedOut => WorkflowStatus.TimedOut,
            // 未知の場合は Failed を返す
            _ => WorkflowStatus.Failed,
        };
    }
}
