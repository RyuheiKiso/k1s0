// k1s0 tier3 4 layer client state reducer（C# .NET 8+ 等価強度実装）
// TypeScript primary と同等の抽象を C# で実装する
// 適合仕様 11_クライアント状態適合仕様.md の v1 layer セットに準拠する
// Phase E: FieldDiff disjoint/intersect 分岐 + ChainIdempotencyKey を追加する

namespace K1s0.Tier3.State;

// FieldDiff は field-level diff を表す record（TypeScript FieldDiff と等価）
public sealed record FieldDiff(
    // client が変更したフィールド名のリスト
    IReadOnlyList<string> ClientFields,
    // server が変更したフィールド名のリスト
    IReadOnlyList<string> ServerFields
)
{
    // IsDisjoint: client と server の変更フィールドが disjoint かどうかを返す
    // true = rebase_clean（auto resend 可能）
    // false = rebase_dirty（3way merge UI 必要）
    public bool IsDisjoint =>
        // client フィールドの HashSet を生成して O(1) 検索を可能にする
        !ServerFields.Any(f => ClientFields.Contains(f));

    // Intersecting: client と server の変更フィールドの交差（共通部分）を返す
    public IReadOnlyList<string> Intersecting =>
        // 交差フィールドを LINQ で取得する
        ServerFields.Where(f => ClientFields.Contains(f)).ToList();
}

// ChainIdempotencyKey: base key から chain された新しい idempotency key を生成する
// TypeScript の chainIdempotencyKey（outbox.ts）と等価の実装
public static class IdempotencyKeyHelper
{
    // ChainIdempotencyKey: base から chain された新しい key を生成する
    // base: 元の idempotency key（chain 親）
    // next: 追加の識別子（aggregate ID + method のハッシュ等）
    public static string ChainIdempotencyKey(string @base, string next)
    {
        // ランダムサフィックスを生成する（Guid で UUID v4 相当）
        var randomSuffix = Guid.NewGuid().ToString("N")[..16];
        // base の先頭 12 文字を prefix に使用する（長すぎる場合は切り詰める）
        var basePrefix = @base.Length > 12 ? @base[..12] : @base;
        // next の先頭 8 文字を prefix に使用する
        var nextPrefix = next.Length > 8 ? next[..8] : next;
        // chain された key を生成する（base_prefix + next_prefix + random_suffix）
        return $"{basePrefix}_{nextPrefix}_{randomSuffix}";
    }
}

// layer を識別する enum
public enum LayerId
{
    // tier2 atomic 三表書込確定値のキャッシュ
    ServerTruth,
    // mutation in-flight overlay
    OptimisticLocal,
    // offline 永続化 mutation 経路
    PendingQueue,
    // 編集中フォームの dirty state
    Draft,
}

// 5 conflict event の基底 record
public abstract record ConflictEvent;
// server_truth バージョン増加
public sealed record ServerTruthAdvanceEvent(long NewVersion) : ConflictEvent;
// mutation in-flight の ack
public sealed record OptimisticAcknowledgedEvent(string IdempotencyKey, long ConfirmedVersion) : ConflictEvent;
// mutation in-flight の reject
public sealed record OptimisticRejectedEvent(string IdempotencyKey, string ErrorCode, BusinessConflictSubtype? ConflictSubtype = null) : ConflictEvent;
// ネットワーク復帰 / アプリ再開
public sealed record PendingQueueResumeEvent(string ResumeReason = "network_recovery") : ConflictEvent;
// api_response_409 + subtype（FieldDiff 付き）
public sealed record BusinessConflictReceivedEvent(
    BusinessConflictSubtype Subtype,
    string AggregateId,
    // field-level diff（stale_write / lost_update 判定に使用、null は safe 側フォールバック）
    FieldDiff? FieldDiff = null
) : ConflictEvent;

// BusinessConflict subtype（4 種固定）
public enum BusinessConflictSubtype
{
    // field disjoint: rebase → auto resend
    StaleWrite,
    // field intersect: 3way merge UI
    LostUpdate,
    // 同 actor 後続 op で既に更新済み
    Supersede,
    // 他 actor presence 中
    ConcurrentEdit,
}

// 5 purge trigger enum
public enum PurgeReason
{
    // ログアウト
    Logout,
    // refresh token 失効
    RefreshTokenExpiry,
    // テナント切り替え
    TenantSwitch,
    // actor 切り替え
    ActorSwitch,
    // device_bound_key ローテーション
    DeviceBoundKeyRotate,
}

// reducer が返す副作用 actions の基底 record
public abstract record ReducerAction;
// server_truth を更新する
public sealed record UpdateServerTruthAction(long Version) : ReducerAction;
// optimistic local を rollback する
public sealed record RollbackOptimisticAction : ReducerAction;
// pending queue を再評価する
public sealed record ReEvaluatePendingQueueAction : ReducerAction;
// optimistic を server_truth に promote する
public sealed record PromoteOptimisticAction(string IdempotencyKey) : ReducerAction;
// pending queue entry を削除する
public sealed record DeletePqEntryAction(string IdempotencyKey) : ReducerAction;
// queue を in-order で送信する
public sealed record SendQueueInOrderAction : ReducerAction;
// business error を表示する
public sealed record PresentBusinessErrorAction(string ErrorCode) : ReducerAction;
// BusinessConflict subtype action を dispatch する
public sealed record DispatchConflictSubtypeAction(BusinessConflictSubtype Subtype) : ReducerAction;
// 3way merge UI を表示する
public sealed record Present3WayMergeUiAction : ReducerAction;
// silent toast を表示する
public sealed record NotifySilentToastAction(string Message) : ReducerAction;
// queue を hold する
public sealed record HoldQueueAction : ReducerAction;
// presence indicator を更新する
public sealed record UpdatePresenceAction(string ActorId) : ReducerAction;
// 全 layer を purge する
public sealed record PurgeAllLayersAction(PurgeReason Reason) : ReducerAction;

// 4 layer client state（immutable record）
public sealed record ClientState(
    // server_truth の aggregate バージョン（null = 未初期化）
    long? ServerTruthVersion = null,
    // optimistic_local の idempotency_key（null = in-flight なし）
    string? OptimisticLocalKey = null,
    // pending_queue の idempotency_key リスト（enqueue 順）
    IReadOnlyList<string>? PendingQueueKeys = null,
    // queue hold 中フラグ
    bool QueueHeld = false
)
{
    // 初期 state を生成する
    public static ClientState Initial() =>
        new(PendingQueueKeys: []);

    // PendingQueueKeys の null safe getter
    public IReadOnlyList<string> SafePendingQueueKeys =>
        PendingQueueKeys ?? [];
}

// reducer の結果
public sealed record ReducerResult(
    // 次の client state
    ClientState NextState,
    // 実行すべき副作用 actions
    IReadOnlyList<ReducerAction> Actions
);

// 4 layer client state reducer
public static class ClientStateReducer
{
    // 全 layer purge（5 trigger）
    public static ReducerResult ReducePurge(ClientState state, PurgeReason reason)
    {
        // 全 layer を purge して初期 state に戻す
        _ = state;
        return new(ClientState.Initial(), [new PurgeAllLayersAction(reason)]);
    }

    // 5 event を処理する決定論的 reducer
    public static ReducerResult Reduce(ClientState state, ConflictEvent ev) =>
        ev switch
        {
            // server_truth_advance: ST 更新 + OL rollback + PQ 再評価
            ServerTruthAdvanceEvent(var ver) => ReduceServerTruthAdvance(state, ver),
            // optimistic_acknowledged: OL → ST promote + PQ entry 削除
            OptimisticAcknowledgedEvent(var key, var ver) => ReduceOptimisticAcknowledged(state, key, ver),
            // optimistic_rejected: OL rollback + business error 表示
            OptimisticRejectedEvent(var key, var code, var subtype) => ReduceOptimisticRejected(state, key, code, subtype),
            // pending_queue_resume: PQ を in-order で送信する
            PendingQueueResumeEvent => ReducePendingQueueResume(state),
            // business_conflict_received: subtype に応じて決定論的に dispatch する（FieldDiff 分岐含む）
            BusinessConflictReceivedEvent(var subtype, _, var fieldDiff) => ReduceBusinessConflict(state, subtype, fieldDiff),
            // 網羅性チェック（新 event 追加時はコンパイルエラー）
            _ => throw new ArgumentOutOfRangeException(nameof(ev), $"Unknown event: {ev.GetType().Name}"),
        };

    // server_truth_advance の reducer
    private static ReducerResult ReduceServerTruthAdvance(ClientState state, long newVersion)
    {
        // OL が存在する場合は rollback する
        var actions = new List<ReducerAction> { new UpdateServerTruthAction(newVersion) };
        var nextState = state;
        if (state.OptimisticLocalKey is not null)
        {
            nextState = nextState with { OptimisticLocalKey = null };
            actions.Add(new RollbackOptimisticAction());
        }
        // PQ 再評価
        actions.Add(new ReEvaluatePendingQueueAction());
        return new(nextState, actions);
    }

    // optimistic_acknowledged の reducer
    private static ReducerResult ReduceOptimisticAcknowledged(ClientState state, string idempotencyKey, long confirmedVersion)
    {
        // OL を null にして PQ から entry を削除する
        var filtered = state.SafePendingQueueKeys.Where(k => k != idempotencyKey).ToList();
        var nextState = state with { OptimisticLocalKey = null, PendingQueueKeys = filtered };
        return new(nextState, [
            new PromoteOptimisticAction(idempotencyKey),
            new DeletePqEntryAction(idempotencyKey),
            new UpdateServerTruthAction(confirmedVersion),
        ]);
    }

    // optimistic_rejected の reducer
    private static ReducerResult ReduceOptimisticRejected(ClientState state, string _, string errorCode, BusinessConflictSubtype? subtype)
    {
        // OL を rollback する
        var nextState = state with { OptimisticLocalKey = null };
        var actions = new List<ReducerAction>
        {
            new RollbackOptimisticAction(),
            new PresentBusinessErrorAction(errorCode),
        };
        // BusinessConflict の場合は subtype dispatch を追加する
        if (subtype is not null)
        {
            actions.Add(new DispatchConflictSubtypeAction(subtype.Value));
        }
        return new(nextState, actions);
    }

    // pending_queue_resume の reducer
    private static ReducerResult ReducePendingQueueResume(ClientState state)
    {
        // PQ が空 / hold 中の場合は何もしない
        if (state.QueueHeld || state.SafePendingQueueKeys.Count == 0)
            return new(state, []);
        return new(state, [new SendQueueInOrderAction()]);
    }

    // business_conflict_received の reducer（FieldDiff 分岐あり）
    private static ReducerResult ReduceBusinessConflict(ClientState state, BusinessConflictSubtype subtype, FieldDiff? fieldDiff = null)
    {
        // subtype に応じて決定論的に actions を返す（分岐 override 禁止）
        return subtype switch
        {
            // stale_write: FieldDiff.IsDisjoint で rebase_clean/dirty を分岐する
            BusinessConflictSubtype.StaleWrite => ReduceStaleWrite(state, fieldDiff),
            // lost_update: safe 側（dirty）に倒して 3way merge UI + hold
            BusinessConflictSubtype.LostUpdate =>
                new(state with { QueueHeld = true }, [new Present3WayMergeUiAction(), new HoldQueueAction()]),
            // supersede: PQ 先頭 entry 削除 + silent toast
            BusinessConflictSubtype.Supersede => ReduceSupersede(state),
            // concurrent_edit: presence 更新
            BusinessConflictSubtype.ConcurrentEdit =>
                new(state, [new UpdatePresenceAction("unknown")]),
            // 網羅性チェック
            _ => throw new ArgumentOutOfRangeException(nameof(subtype)),
        };
    }

    // stale_write の reducer（FieldDiff.IsDisjoint で rebase_clean/dirty を分岐する）
    private static ReducerResult ReduceStaleWrite(ClientState state, FieldDiff? fieldDiff)
    {
        // FieldDiff が存在し disjoint の場合は rebase_clean → auto resend
        if (fieldDiff is { IsDisjoint: true })
        {
            // OL の idempotency key を取得して chain する
            var baseKey = state.OptimisticLocalKey ?? "unknown";
            // chain された新しい idempotency key を生成する
            var newKey = IdempotencyKeyHelper.ChainIdempotencyKey(baseKey, "rebase");
            // OL を rollback して auto resend する
            var nextState = state with { OptimisticLocalKey = null };
            return new(nextState, [
                new RollbackOptimisticAction(),
                new SendQueueInOrderAction(),
                // chain した新 key を detail に含む silent toast で通知する
                new NotifySilentToastAction($"rebase_clean: auto resend with key={newKey}"),
            ]);
        }
        // FieldDiff がない / intersecting の場合は rebase_dirty → 3way merge UI + hold
        return new(state with { QueueHeld = true }, [new Present3WayMergeUiAction(), new HoldQueueAction()]);
    }

    // supersede の reducer
    private static ReducerResult ReduceSupersede(ClientState state)
    {
        // PQ 先頭 entry を削除する
        var keys = state.SafePendingQueueKeys.ToList();
        var actions = new List<ReducerAction>();
        if (keys.Count > 0)
        {
            var key = keys[0];
            keys.RemoveAt(0);
            actions.Add(new DeletePqEntryAction(key));
        }
        // silent toast を追加する
        actions.Add(new NotifySilentToastAction("後続の操作で既に上書きされました"));
        return new(state with { PendingQueueKeys = keys }, actions);
    }
}
