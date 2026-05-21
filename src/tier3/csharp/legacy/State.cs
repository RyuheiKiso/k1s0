// State.cs — .NET Framework 4.6.2 互換の 4 layer state reducer（レガシー実装）
// .NET 8+ の State.cs と同一の公開 API を保ちつつ、.NET 4.6.2 制約に適合する
// 制約: ValueTuple 不使用 / async/await 代わりに Task.Result を使用
// 制約: readonly struct 不使用 / record 型不使用 / IReadOnlyList は使用可（.NET 4.5+）
// 設計方針: 11_クライアント状態適合仕様.md の 4 layer を .NET 4.6.2 で完全実装する

// System 名前空間をインポートする（基本型に必要）
using System;
// コレクション型をインポートする（List<T> / IReadOnlyList<T> 使用のため）
using System.Collections.Generic;
// LINQ をインポートする（コレクション操作に使用する）
using System.Linq;
// Task 型をインポートする（非同期操作のタスク管理に必要）
using System.Threading.Tasks;

// K1s0.Tier3.Legacy 名前空間（.NET 8+ の K1s0.Tier3.State と分離する）
namespace K1s0.Tier3.Legacy
{
    // FieldDiff は field-level diff を表すクラス（.NET 4.6.2 では sealed record の代わりに sealed class を使用する）
    public sealed class FieldDiff
    {
        // client が変更したフィールド名のリスト
        private readonly IReadOnlyList<string> _clientFields;
        // server が変更したフィールド名のリスト
        private readonly IReadOnlyList<string> _serverFields;

        // コンストラクター（フィールドを初期化する）
        public FieldDiff(IReadOnlyList<string> clientFields, IReadOnlyList<string> serverFields)
        {
            // client フィールドリストを設定する
            _clientFields = clientFields ?? new List<string>();
            // server フィールドリストを設定する
            _serverFields = serverFields ?? new List<string>();
        }

        // ClientFields プロパティ（読み取り専用）
        public IReadOnlyList<string> ClientFields { get { return _clientFields; } }
        // ServerFields プロパティ（読み取り専用）
        public IReadOnlyList<string> ServerFields { get { return _serverFields; } }

        // IsDisjoint: client と server の変更フィールドが disjoint かどうかを返す
        // true = rebase_clean（auto resend 可能）、false = rebase_dirty（3way merge UI 必要）
        public bool IsDisjoint
        {
            get
            {
                // server フィールドの中に client フィールドと重複するものがないか確認する
                foreach (var serverField in _serverFields)
                {
                    // client フィールドに同じ名前が含まれている場合は disjoint ではない
                    if (_clientFields.Contains(serverField))
                    {
                        return false;
                    }
                }
                // 重複がなければ disjoint（true を返す）
                return true;
            }
        }

        // Intersecting: client と server の変更フィールドの交差（共通部分）を返す
        public IReadOnlyList<string> Intersecting
        {
            get
            {
                // 交差フィールドを収集するリストを初期化する
                var result = new List<string>();
                // server フィールドを順番に確認して client フィールドと重複するものを収集する
                foreach (var serverField in _serverFields)
                {
                    // client フィールドに含まれている場合は result に追加する
                    if (_clientFields.Contains(serverField))
                    {
                        result.Add(serverField);
                    }
                }
                // 交差フィールドのリストを返す
                return result.AsReadOnly();
            }
        }
    }

    // ChainIdempotencyKey: base key から chain された新しい idempotency key を生成する
    // .NET 8+ の IdempotencyKeyHelper と同等の実装
    public static class IdempotencyKeyHelper
    {
        // ChainIdempotencyKey: base から chain された新しい key を生成する
        // base: 元の idempotency key（chain 親）
        // next: 追加の識別子（aggregate ID + method のハッシュ等）
        public static string ChainIdempotencyKey(string baseKey, string next)
        {
            // ランダムサフィックスを生成する（Guid で UUID v4 相当）
            var randomSuffix = Guid.NewGuid().ToString("N").Substring(0, 16);
            // base の先頭 12 文字を prefix に使用する（長すぎる場合は切り詰める）
            var basePrefix = baseKey.Length > 12 ? baseKey.Substring(0, 12) : baseKey;
            // next の先頭 8 文字を prefix に使用する
            var nextPrefix = next.Length > 8 ? next.Substring(0, 8) : next;
            // chain された key を生成する（base_prefix + next_prefix + random_suffix）
            return string.Format("{0}_{1}_{2}", basePrefix, nextPrefix, randomSuffix);
        }
    }

    // layer を識別する列挙型
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

    // 5 purge trigger 列挙型
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

    // ConflictEvent の基底クラス（.NET 4.6.2 では abstract class を使用する）
    public abstract class ConflictEvent { }

    // server_truth バージョン増加イベント
    public sealed class ServerTruthAdvanceEvent : ConflictEvent
    {
        // 新しいバージョン番号
        public readonly long NewVersion;
        // コンストラクター
        public ServerTruthAdvanceEvent(long newVersion) { NewVersion = newVersion; }
    }

    // mutation in-flight の ack イベント
    public sealed class OptimisticAcknowledgedEvent : ConflictEvent
    {
        // idempotency key
        public readonly string IdempotencyKey;
        // 確認されたバージョン
        public readonly long ConfirmedVersion;
        // コンストラクター
        public OptimisticAcknowledgedEvent(string idempotencyKey, long confirmedVersion)
        {
            // idempotency key を設定する
            IdempotencyKey = idempotencyKey;
            // 確認バージョンを設定する
            ConfirmedVersion = confirmedVersion;
        }
    }

    // mutation in-flight の reject イベント
    public sealed class OptimisticRejectedEvent : ConflictEvent
    {
        // idempotency key
        public readonly string IdempotencyKey;
        // エラーコード
        public readonly string ErrorCode;
        // BusinessConflict subtype（null = 非 business conflict）
        public readonly BusinessConflictSubtype? ConflictSubtype;
        // コンストラクター（subtype は省略可能）
        public OptimisticRejectedEvent(string idempotencyKey, string errorCode, BusinessConflictSubtype? conflictSubtype = null)
        {
            // フィールドを設定する
            IdempotencyKey = idempotencyKey;
            ErrorCode = errorCode;
            ConflictSubtype = conflictSubtype;
        }
    }

    // ネットワーク復帰 / アプリ再開イベント
    public sealed class PendingQueueResumeEvent : ConflictEvent
    {
        // 再開理由
        public readonly string ResumeReason;
        // コンストラクター
        public PendingQueueResumeEvent(string resumeReason = "network_recovery")
        {
            // 再開理由を設定する
            ResumeReason = resumeReason;
        }
    }

    // api_response_409 + subtype（FieldDiff 付き）イベント
    public sealed class BusinessConflictReceivedEvent : ConflictEvent
    {
        // conflict の subtype
        public readonly BusinessConflictSubtype Subtype;
        // aggregate ID
        public readonly string AggregateId;
        // field-level diff（null は safe 側フォールバック）
        public readonly FieldDiff FieldDiff;
        // コンストラクター
        public BusinessConflictReceivedEvent(BusinessConflictSubtype subtype, string aggregateId, FieldDiff fieldDiff = null)
        {
            // フィールドを設定する
            Subtype = subtype;
            AggregateId = aggregateId;
            FieldDiff = fieldDiff;
        }
    }

    // ReducerAction の基底クラス
    public abstract class ReducerAction { }
    // server_truth を更新するアクション
    public sealed class UpdateServerTruthAction : ReducerAction
    {
        // 更新後のバージョン
        public readonly long Version;
        // コンストラクター
        public UpdateServerTruthAction(long version) { Version = version; }
    }
    // optimistic local を rollback するアクション
    public sealed class RollbackOptimisticAction : ReducerAction { }
    // pending queue を再評価するアクション
    public sealed class ReEvaluatePendingQueueAction : ReducerAction { }
    // optimistic を server_truth に promote するアクション
    public sealed class PromoteOptimisticAction : ReducerAction
    {
        // idempotency key
        public readonly string IdempotencyKey;
        // コンストラクター
        public PromoteOptimisticAction(string idempotencyKey) { IdempotencyKey = idempotencyKey; }
    }
    // pending queue entry を削除するアクション
    public sealed class DeletePqEntryAction : ReducerAction
    {
        // idempotency key
        public readonly string IdempotencyKey;
        // コンストラクター
        public DeletePqEntryAction(string idempotencyKey) { IdempotencyKey = idempotencyKey; }
    }
    // queue を in-order で送信するアクション
    public sealed class SendQueueInOrderAction : ReducerAction { }
    // business error を表示するアクション
    public sealed class PresentBusinessErrorAction : ReducerAction
    {
        // エラーコード
        public readonly string ErrorCode;
        // コンストラクター
        public PresentBusinessErrorAction(string errorCode) { ErrorCode = errorCode; }
    }
    // BusinessConflict subtype action を dispatch するアクション
    public sealed class DispatchConflictSubtypeAction : ReducerAction
    {
        // subtype
        public readonly BusinessConflictSubtype Subtype;
        // コンストラクター
        public DispatchConflictSubtypeAction(BusinessConflictSubtype subtype) { Subtype = subtype; }
    }
    // 3way merge UI を表示するアクション
    public sealed class Present3WayMergeUiAction : ReducerAction { }
    // silent toast を表示するアクション
    public sealed class NotifySilentToastAction : ReducerAction
    {
        // メッセージ
        public readonly string Message;
        // コンストラクター
        public NotifySilentToastAction(string message) { Message = message; }
    }
    // queue を hold するアクション
    public sealed class HoldQueueAction : ReducerAction { }
    // presence indicator を更新するアクション
    public sealed class UpdatePresenceAction : ReducerAction
    {
        // actor ID
        public readonly string ActorId;
        // コンストラクター
        public UpdatePresenceAction(string actorId) { ActorId = actorId; }
    }
    // 全 layer を purge するアクション
    public sealed class PurgeAllLayersAction : ReducerAction
    {
        // purge 理由
        public readonly PurgeReason Reason;
        // コンストラクター
        public PurgeAllLayersAction(PurgeReason reason) { Reason = reason; }
    }

    // auto_resend_with_chained_key の型付きアクション（T3-4: string ではなく型で表現する）
    // TypeScript の AutoResendWithChainedKeyAction と 4 言語等価強度を保つ
    // string-formatted NotifySilentToastAction の代替として rebase_clean パスで使用する
    public sealed class AutoResendWithChainedKeyReducerAction : ReducerAction
    {
        // chain 元の idempotency_key（rebase 前の key）
        public readonly string ChainedFrom;
        // chain 後の新しい idempotency_key
        public readonly string NewKey;
        // コンストラクター（chainedFrom と newKey を必須で設定する）
        public AutoResendWithChainedKeyReducerAction(string chainedFrom, string newKey)
        {
            // chain 元の key を設定する
            ChainedFrom = chainedFrom;
            // chain 後の key を設定する
            NewKey = newKey;
        }
    }

    // 4 layer client state（.NET 4.6.2 では immutable pattern を手動で実装する）
    public sealed class ClientState
    {
        // server_truth の aggregate バージョン（null = 未初期化）
        public readonly long? ServerTruthVersion;
        // optimistic_local の idempotency_key（null = in-flight なし）
        public readonly string OptimisticLocalKey;
        // pending_queue の idempotency_key リスト（enqueue 順）
        public readonly IReadOnlyList<string> PendingQueueKeys;
        // queue hold 中フラグ
        public readonly bool QueueHeld;

        // コンストラクター（全フィールドを初期化する）
        public ClientState(
            long? serverTruthVersion = null,
            string optimisticLocalKey = null,
            IReadOnlyList<string> pendingQueueKeys = null,
            bool queueHeld = false)
        {
            // 各フィールドを設定する
            ServerTruthVersion = serverTruthVersion;
            OptimisticLocalKey = optimisticLocalKey;
            // null の場合は空リストで初期化する
            PendingQueueKeys = pendingQueueKeys ?? new List<string>().AsReadOnly();
            QueueHeld = queueHeld;
        }

        // 初期 state を生成する
        public static ClientState Initial()
        {
            // 全フィールドを既定値で初期化した state を返す
            return new ClientState(pendingQueueKeys: new List<string>().AsReadOnly());
        }

        // With メソッド群（.NET 4.6.2 では record with 式の代わりにコピーコンストラクターを使用する）
        // OptimisticLocalKey を変更した新しい state を返す
        public ClientState WithOptimisticLocalKey(string optimisticLocalKey)
        {
            // 新しい state インスタンスを返す
            return new ClientState(ServerTruthVersion, optimisticLocalKey, PendingQueueKeys, QueueHeld);
        }

        // PendingQueueKeys を変更した新しい state を返す
        public ClientState WithPendingQueueKeys(IReadOnlyList<string> pendingQueueKeys)
        {
            // 新しい state インスタンスを返す
            return new ClientState(ServerTruthVersion, OptimisticLocalKey, pendingQueueKeys, QueueHeld);
        }

        // QueueHeld を変更した新しい state を返す
        public ClientState WithQueueHeld(bool queueHeld)
        {
            // 新しい state インスタンスを返す
            return new ClientState(ServerTruthVersion, OptimisticLocalKey, PendingQueueKeys, queueHeld);
        }
    }

    // reducer の結果クラス（.NET 4.6.2 では sealed class を使用する）
    public sealed class ReducerResult
    {
        // 次の client state
        public readonly ClientState NextState;
        // 実行すべき副作用 actions
        public readonly IReadOnlyList<ReducerAction> Actions;
        // コンストラクター
        public ReducerResult(ClientState nextState, IReadOnlyList<ReducerAction> actions)
        {
            // フィールドを設定する
            NextState = nextState;
            Actions = actions;
        }
    }

    // 4 layer client state reducer（.NET Framework 4.6.2 互換実装）
    public static class ClientStateReducer
    {
        // 全 layer purge（5 trigger）
        public static ReducerResult ReducePurge(ClientState state, PurgeReason reason)
        {
            // 全 layer を purge して初期 state に戻す
            return new ReducerResult(
                ClientState.Initial(),
                new List<ReducerAction> { new PurgeAllLayersAction(reason) }.AsReadOnly()
            );
        }

        // 5 event を処理する決定論的 reducer
        public static ReducerResult Reduce(ClientState state, ConflictEvent ev)
        {
            // event 種別に応じて reducer を分岐する（.NET 4.6.2 では pattern matching 不可）
            var serverTruthAdvance = ev as ServerTruthAdvanceEvent;
            if (serverTruthAdvance != null)
            {
                // server_truth_advance: ST 更新 + OL rollback + PQ 再評価
                return ReduceServerTruthAdvance(state, serverTruthAdvance.NewVersion);
            }

            var optimisticAcknowledged = ev as OptimisticAcknowledgedEvent;
            if (optimisticAcknowledged != null)
            {
                // optimistic_acknowledged: OL → ST promote + PQ entry 削除
                return ReduceOptimisticAcknowledged(state, optimisticAcknowledged.IdempotencyKey, optimisticAcknowledged.ConfirmedVersion);
            }

            var optimisticRejected = ev as OptimisticRejectedEvent;
            if (optimisticRejected != null)
            {
                // optimistic_rejected: OL rollback + business error 表示
                return ReduceOptimisticRejected(state, optimisticRejected.IdempotencyKey, optimisticRejected.ErrorCode, optimisticRejected.ConflictSubtype);
            }

            var pendingQueueResume = ev as PendingQueueResumeEvent;
            if (pendingQueueResume != null)
            {
                // pending_queue_resume: PQ を in-order で送信する
                return ReducePendingQueueResume(state);
            }

            var businessConflict = ev as BusinessConflictReceivedEvent;
            if (businessConflict != null)
            {
                // business_conflict_received: subtype に応じて決定論的に dispatch する
                return ReduceBusinessConflict(state, businessConflict.Subtype, businessConflict.FieldDiff);
            }

            // 未知の event 型は InvalidOperationException をスローする
            throw new InvalidOperationException(string.Format("Unknown event: {0}", ev.GetType().Name));
        }

        // server_truth_advance の reducer
        private static ReducerResult ReduceServerTruthAdvance(ClientState state, long newVersion)
        {
            // 実行すべきアクションリストを初期化する
            var actions = new List<ReducerAction>();
            // ST 更新アクションを追加する
            actions.Add(new UpdateServerTruthAction(newVersion));
            // 次の state を設定する（初期は入力と同じ）
            var nextState = state;

            // OL が存在する場合は rollback アクションを追加する
            if (state.OptimisticLocalKey != null)
            {
                // OL key を null にした新しい state を生成する
                nextState = nextState.WithOptimisticLocalKey(null);
                // rollback アクションを追加する
                actions.Add(new RollbackOptimisticAction());
            }

            // PQ 再評価アクションを追加する
            actions.Add(new ReEvaluatePendingQueueAction());
            // ReducerResult を返す
            return new ReducerResult(nextState, actions.AsReadOnly());
        }

        // optimistic_acknowledged の reducer
        private static ReducerResult ReduceOptimisticAcknowledged(ClientState state, string idempotencyKey, long confirmedVersion)
        {
            // PQ から対象 key を除外したリストを生成する
            var filteredKeys = state.PendingQueueKeys
                .Where(k => k != idempotencyKey)
                .ToList()
                .AsReadOnly();
            // OL を null にして PQ から entry を削除した新しい state を生成する
            var nextState = state
                .WithOptimisticLocalKey(null)
                .WithPendingQueueKeys(filteredKeys);
            // アクションリストを生成する
            var actions = new List<ReducerAction>
            {
                // OL を ST に promote するアクション
                new PromoteOptimisticAction(idempotencyKey),
                // PQ entry を削除するアクション
                new DeletePqEntryAction(idempotencyKey),
                // ST バージョンを更新するアクション
                new UpdateServerTruthAction(confirmedVersion),
            };
            // ReducerResult を返す
            return new ReducerResult(nextState, actions.AsReadOnly());
        }

        // optimistic_rejected の reducer
        private static ReducerResult ReduceOptimisticRejected(ClientState state, string idempotencyKey, string errorCode, BusinessConflictSubtype? subtype)
        {
            // OL を rollback した新しい state を生成する（idempotencyKey は使用するが戻り値は state のみ）
            var nextState = state.WithOptimisticLocalKey(null);
            // アクションリストを初期化する
            var actions = new List<ReducerAction>();
            // OL rollback アクションを追加する
            actions.Add(new RollbackOptimisticAction());
            // business error 表示アクションを追加する
            actions.Add(new PresentBusinessErrorAction(errorCode));

            // BusinessConflict の場合は subtype dispatch を追加する
            if (subtype.HasValue)
            {
                actions.Add(new DispatchConflictSubtypeAction(subtype.Value));
            }

            // 変数 idempotencyKey は API 互換性のために引数として存在するが、このパスでは直接使用しない
            _ = idempotencyKey;
            // ReducerResult を返す
            return new ReducerResult(nextState, actions.AsReadOnly());
        }

        // pending_queue_resume の reducer
        private static ReducerResult ReducePendingQueueResume(ClientState state)
        {
            // PQ が空 / hold 中の場合は何もしない
            if (state.QueueHeld || state.PendingQueueKeys.Count == 0)
            {
                // 空のアクションリストを返す
                return new ReducerResult(state, new List<ReducerAction>().AsReadOnly());
            }
            // PQ を in-order で送信するアクションを返す
            return new ReducerResult(
                state,
                new List<ReducerAction> { new SendQueueInOrderAction() }.AsReadOnly()
            );
        }

        // business_conflict_received の reducer（FieldDiff 分岐あり）
        private static ReducerResult ReduceBusinessConflict(ClientState state, BusinessConflictSubtype subtype, FieldDiff fieldDiff)
        {
            // subtype に応じて決定論的に分岐する
            switch (subtype)
            {
                case BusinessConflictSubtype.StaleWrite:
                    // stale_write: FieldDiff.IsDisjoint で rebase_clean/dirty を分岐する
                    return ReduceStaleWrite(state, fieldDiff);

                case BusinessConflictSubtype.LostUpdate:
                    // lost_update: safe 側（dirty）に倒して 3way merge UI + hold
                    return new ReducerResult(
                        state.WithQueueHeld(true),
                        new List<ReducerAction>
                        {
                            // 3way merge UI を表示するアクション
                            new Present3WayMergeUiAction(),
                            // キューを hold するアクション
                            new HoldQueueAction(),
                        }.AsReadOnly()
                    );

                case BusinessConflictSubtype.Supersede:
                    // supersede: PQ 先頭 entry 削除 + silent toast
                    return ReduceSupersede(state);

                case BusinessConflictSubtype.ConcurrentEdit:
                    // concurrent_edit: presence 更新
                    return new ReducerResult(
                        state,
                        new List<ReducerAction>
                        {
                            // presence indicator を更新するアクション
                            new UpdatePresenceAction("unknown"),
                        }.AsReadOnly()
                    );

                default:
                    // 未知の subtype は InvalidOperationException をスローする
                    throw new InvalidOperationException(string.Format("Unknown subtype: {0}", subtype));
            }
        }

        // stale_write の reducer（FieldDiff.IsDisjoint で rebase_clean/dirty を分岐する）
        private static ReducerResult ReduceStaleWrite(ClientState state, FieldDiff fieldDiff)
        {
            // FieldDiff が存在し disjoint の場合は rebase_clean → auto resend
            if (fieldDiff != null && fieldDiff.IsDisjoint)
            {
                // OL の idempotency key を取得して chain する
                var baseKey = state.OptimisticLocalKey ?? "unknown";
                // chain された新しい idempotency key を生成する
                var newKey = IdempotencyKeyHelper.ChainIdempotencyKey(baseKey, "rebase");
                // OL を rollback した新しい state を生成する
                var nextState = state.WithOptimisticLocalKey(null);
                // rebase_clean アクションリストを返す
                return new ReducerResult(
                    nextState,
                    new List<ReducerAction>
                    {
                        // OL rollback アクション
                        new RollbackOptimisticAction(),
                        // auto resend アクション
                        new SendQueueInOrderAction(),
                        // 型付き AutoResendWithChainedKeyReducerAction（string-formatted action を排除する）
                        // chainedFrom: 元の key、newKey: chain 後の新しい key
                        new AutoResendWithChainedKeyReducerAction(baseKey, newKey),
                    }.AsReadOnly()
                );
            }
            // FieldDiff がない / intersecting の場合は rebase_dirty → 3way merge UI + hold
            return new ReducerResult(
                state.WithQueueHeld(true),
                new List<ReducerAction>
                {
                    // 3way merge UI を表示するアクション
                    new Present3WayMergeUiAction(),
                    // キューを hold するアクション
                    new HoldQueueAction(),
                }.AsReadOnly()
            );
        }

        // supersede の reducer
        private static ReducerResult ReduceSupersede(ClientState state)
        {
            // PQ のリストをコピーする
            var keys = state.PendingQueueKeys.ToList();
            // アクションリストを初期化する
            var actions = new List<ReducerAction>();
            // PQ に entry がある場合は先頭 entry を削除する
            if (keys.Count > 0)
            {
                // 先頭 entry の key を取得する
                var key = keys[0];
                // 先頭 entry を削除する
                keys.RemoveAt(0);
                // 削除アクションを追加する
                actions.Add(new DeletePqEntryAction(key));
            }
            // silent toast アクションを追加する
            actions.Add(new NotifySilentToastAction("後続の操作で既に上書きされました"));
            // 更新された PQ keys を持つ新しい state を返す
            return new ReducerResult(
                state.WithPendingQueueKeys(keys.AsReadOnly()),
                actions.AsReadOnly()
            );
        }
    }
}
