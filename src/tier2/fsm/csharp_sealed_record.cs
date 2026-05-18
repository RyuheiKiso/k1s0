// k1s0 tier2 FSM: C# sealed record による状態遷移型エンコーディング
// fsm_spec.yaml の OrderStatus / BatchStatus を C# の sealed record で表現する
// 不正な状態遷移は compile-time エラーで検出する（状態遷移パターン 13）

// 基本型
using System;

// k1s0 tier2 FSM 名前空間
namespace K1s0.Tier2.Fsm;

// ============================================================
// OrderStatus FSM: オーダーの状態遷移
// ============================================================

/// <summary>
/// IOrderState: オーダー状態のマーカーインターフェース
/// 許容状態型だけが実装できる（sealed record で具象化する）
/// </summary>
public interface IOrderState { }

/// <summary>
/// Draft 状態: 下書き、未確定
/// </summary>
// sealed record で継承を禁止し、イミュータブルな状態型を保証する
public sealed record DraftState : IOrderState;

/// <summary>
/// Submitted 状態: 提出済み、確定
/// </summary>
public sealed record SubmittedState : IOrderState;

/// <summary>
/// Processing 状態: 処理中
/// </summary>
public sealed record ProcessingState : IOrderState;

/// <summary>
/// Completed 状態: 完了、終端状態
/// </summary>
public sealed record CompletedState : IOrderState;

/// <summary>
/// Cancelled 状態: 中止、終端状態
/// </summary>
public sealed record CancelledState : IOrderState;

/// <summary>
/// Order&lt;TState&gt;: 状態を型パラメータで保持するオーダー型
/// 不正遷移は where 制約と sealed メソッドで compile-time に検出する
/// </summary>
public sealed record Order<TState>(
    // オーダーの主キー
    string Id,
    // 現在の状態（型パラメータで静的に保証する）
    TState State
) where TState : IOrderState;

/// <summary>
/// OrderTransitions: オーダー状態遷移の拡張メソッド群
/// 各遷移は型制約により許可された状態からのみ呼べる
/// </summary>
public static class OrderTransitions
{
    /// <summary>
    /// Draft → Submitted: 提出操作（Submit イベント）
    /// Order&lt;DraftState&gt; からのみ呼べる型制約を持つ
    /// </summary>
    public static Order<SubmittedState> Submit(this Order<DraftState> order)
    {
        // 同一 Id で Submitted 状態の Order を返す（状態遷移を型で表現する）
        return new Order<SubmittedState>(order.Id, new SubmittedState());
    }

    /// <summary>
    /// Draft → Cancelled: キャンセル操作（Cancel イベント）
    /// </summary>
    public static Order<CancelledState> Cancel(this Order<DraftState> order)
    {
        // 同一 Id で Cancelled 状態の Order を返す
        return new Order<CancelledState>(order.Id, new CancelledState());
    }

    /// <summary>
    /// Submitted → Processing: 処理開始操作（StartProcessing イベント）
    /// Order&lt;SubmittedState&gt; からのみ呼べる型制約を持つ
    /// </summary>
    public static Order<ProcessingState> StartProcessing(this Order<SubmittedState> order)
    {
        // 同一 Id で Processing 状態の Order を返す
        return new Order<ProcessingState>(order.Id, new ProcessingState());
    }

    /// <summary>
    /// Submitted → Cancelled: 処理開始前のキャンセル操作
    /// </summary>
    public static Order<CancelledState> Cancel(this Order<SubmittedState> order)
    {
        // 同一 Id で Cancelled 状態の Order を返す
        return new Order<CancelledState>(order.Id, new CancelledState());
    }

    /// <summary>
    /// Processing → Completed: 処理完了操作（Complete イベント）
    /// Order&lt;ProcessingState&gt; からのみ呼べる型制約を持つ
    /// </summary>
    public static Order<CompletedState> Complete(this Order<ProcessingState> order)
    {
        // 同一 Id で Completed 状態の Order を返す
        return new Order<CompletedState>(order.Id, new CompletedState());
    }

    /// <summary>
    /// Processing → Cancelled: 処理中キャンセル操作
    /// </summary>
    public static Order<CancelledState> Cancel(this Order<ProcessingState> order)
    {
        // 同一 Id で Cancelled 状態の Order を返す
        return new Order<CancelledState>(order.Id, new CancelledState());
    }
}

// ============================================================
// BatchStatus FSM: バッチ処理の状態遷移
// ============================================================

/// <summary>
/// IBatchState: バッチ状態のマーカーインターフェース
/// </summary>
public interface IBatchState { }

/// <summary>
/// Registered 状態: 登録済み
/// </summary>
public sealed record RegisteredState : IBatchState;

/// <summary>
/// Running 状態: 実行中
/// </summary>
public sealed record RunningState : IBatchState;

/// <summary>
/// Paused 状態: 一時停止
/// </summary>
public sealed record PausedState : IBatchState;

/// <summary>
/// Succeeded 状態: 成功、終端状態
/// </summary>
public sealed record SucceededState : IBatchState;

/// <summary>
/// Failed 状態: 失敗、終端状態
/// </summary>
public sealed record FailedState : IBatchState;

/// <summary>
/// Batch&lt;TState&gt;: バッチ処理の状態を型パラメータで保持する型
/// </summary>
public sealed record Batch<TState>(
    // バッチの主キー
    string Id,
    // 現在の状態（型パラメータで静的に保証する）
    TState State
) where TState : IBatchState;

/// <summary>
/// BatchTransitions: バッチ状態遷移の拡張メソッド群
/// </summary>
public static class BatchTransitions
{
    /// <summary>
    /// Registered → Running: バッチ実行開始操作（Start イベント）
    /// </summary>
    public static Batch<RunningState> Start(this Batch<RegisteredState> batch)
    {
        // 同一 Id で Running 状態の Batch を返す
        return new Batch<RunningState>(batch.Id, new RunningState());
    }

    /// <summary>
    /// Running → Paused: 一時停止操作（Pause イベント）
    /// </summary>
    public static Batch<PausedState> Pause(this Batch<RunningState> batch)
    {
        // 同一 Id で Paused 状態の Batch を返す
        return new Batch<PausedState>(batch.Id, new PausedState());
    }

    /// <summary>
    /// Running → Succeeded: 正常終了操作（Succeed イベント）
    /// </summary>
    public static Batch<SucceededState> Succeed(this Batch<RunningState> batch)
    {
        // 同一 Id で Succeeded 状態の Batch を返す
        return new Batch<SucceededState>(batch.Id, new SucceededState());
    }

    /// <summary>
    /// Running → Failed: 異常終了操作（Fail イベント）
    /// </summary>
    public static Batch<FailedState> Fail(this Batch<RunningState> batch)
    {
        // 同一 Id で Failed 状態の Batch を返す
        return new Batch<FailedState>(batch.Id, new FailedState());
    }

    /// <summary>
    /// Paused → Running: 再開操作（Resume イベント）
    /// </summary>
    public static Batch<RunningState> Resume(this Batch<PausedState> batch)
    {
        // 同一 Id で Running 状態の Batch を返す
        return new Batch<RunningState>(batch.Id, new RunningState());
    }

    /// <summary>
    /// Paused → Failed: 一時停止中の異常終了（Fail イベント）
    /// </summary>
    public static Batch<FailedState> Fail(this Batch<PausedState> batch)
    {
        // 同一 Id で Failed 状態の Batch を返す
        return new Batch<FailedState>(batch.Id, new FailedState());
    }
}
