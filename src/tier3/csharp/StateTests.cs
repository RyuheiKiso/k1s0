// k1s0 tier3 C# state reducer テスト（5 event × 4 subtype 等価強度検証）
// TypeScript primary と同等の決定論的 reducer 挙動を C# で検証する
// 適合仕様 11_クライアント状態適合仕様.md の v1 layer セットに準拠する

using Microsoft.VisualStudio.TestTools.UnitTesting;

// テスト対象名前空間をインポートする
namespace K1s0.Tier3.State;

// reducer の 5 event × 4 subtype を検証するテストクラス
[TestClass]
public class ReducerTests
{
    // ---------------------------------------------------------
    // TestServerTruthAdvance_RollbacksOptimistic
    // ---------------------------------------------------------
    [TestMethod]
    public void TestServerTruthAdvance_RollbacksOptimistic()
    {
        // OL が存在する state で server_truth_advance を処理する
        var state = ClientState.Initial() with { OptimisticLocalKey = "idem-001" };
        // version=5 の server_truth_advance event を発行する
        var result = ClientStateReducer.Reduce(state, new ServerTruthAdvanceEvent(5));
        // next state で OL が null になることを確認する
        Assert.IsNull(result.NextState.OptimisticLocalKey, "OL は rollback で null になること");
        // UPDATE_SERVER_TRUTH action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<UpdateServerTruthAction>().Any(a => a.Version == 5),
            "UpdateServerTruth(5) が含まれること");
        // ROLLBACK_OPTIMISTIC action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<RollbackOptimisticAction>().Any(),
            "RollbackOptimistic が含まれること");
        // RE_EVALUATE_PENDING_QUEUE action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<ReEvaluatePendingQueueAction>().Any(),
            "ReEvaluatePendingQueue が含まれること");
    }

    // ---------------------------------------------------------
    // TestOptimisticAcknowledged_PromotesToST
    // ---------------------------------------------------------
    [TestMethod]
    public void TestOptimisticAcknowledged_PromotesToST()
    {
        // PQ に "idem-ack" が存在する state を用意する
        var state = ClientState.Initial() with
        {
            OptimisticLocalKey = "idem-ack",
            PendingQueueKeys = new List<string> { "idem-ack" },
        };
        // optimistic_acknowledged event（version=10）を発行する
        var result = ClientStateReducer.Reduce(state, new OptimisticAcknowledgedEvent("idem-ack", 10));
        // next state で OL が null になることを確認する
        Assert.IsNull(result.NextState.OptimisticLocalKey, "OL は promoted で null になること");
        // PQ から entry が削除されていることを確認する
        Assert.IsFalse(
            result.NextState.SafePendingQueueKeys.Contains("idem-ack"),
            "PQ から idem-ack が削除されること");
        // PROMOTE_OPTIMISTIC action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<PromoteOptimisticAction>().Any(a => a.IdempotencyKey == "idem-ack"),
            "PromoteOptimistic(idem-ack) が含まれること");
        // UPDATE_SERVER_TRUTH(10) action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<UpdateServerTruthAction>().Any(a => a.Version == 10),
            "UpdateServerTruth(10) が含まれること");
    }

    // ---------------------------------------------------------
    // TestBusinessConflict_StaleWrite_AutoResend
    // ---------------------------------------------------------
    [TestMethod]
    public void TestBusinessConflict_StaleWrite_AutoResend()
    {
        // 初期 state で stale_write conflict を受け取る
        var state = ClientState.Initial();
        // BusinessConflictReceived(StaleWrite) event を発行する
        var result = ClientStateReducer.Reduce(
            state,
            new BusinessConflictReceivedEvent(BusinessConflictSubtype.StaleWrite, "agg-sw-001"));
        // next state で QueueHeld が true になることを確認する
        Assert.IsTrue(result.NextState.QueueHeld, "QueueHeld が true になること");
        // Present3WayMergeUi action が含まれることを確認する（safe 側に倒す設計）
        Assert.IsTrue(
            result.Actions.OfType<Present3WayMergeUiAction>().Any(),
            "Present3WayMergeUi が含まれること");
        // HoldQueue action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<HoldQueueAction>().Any(),
            "HoldQueue が含まれること");
    }

    // ---------------------------------------------------------
    // TestBusinessConflict_LostUpdate_3wayMerge
    // ---------------------------------------------------------
    [TestMethod]
    public void TestBusinessConflict_LostUpdate_3wayMerge()
    {
        // 初期 state で lost_update conflict を受け取る
        var state = ClientState.Initial();
        // BusinessConflictReceived(LostUpdate) event を発行する
        var result = ClientStateReducer.Reduce(
            state,
            new BusinessConflictReceivedEvent(BusinessConflictSubtype.LostUpdate, "agg-lu-001"));
        // next state で QueueHeld が true になることを確認する
        Assert.IsTrue(result.NextState.QueueHeld, "QueueHeld が true になること");
        // Present3WayMergeUi action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<Present3WayMergeUiAction>().Any(),
            "Present3WayMergeUi が含まれること（lost_update は 3way merge UI 表示）");
        // HoldQueue action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<HoldQueueAction>().Any(),
            "HoldQueue が含まれること");
    }

    // ---------------------------------------------------------
    // TestPurge_ClearsAllLayers
    // ---------------------------------------------------------
    [TestMethod]
    public void TestPurge_ClearsAllLayers()
    {
        // 全 layer に値が入った state を用意する
        var state = ClientState.Initial() with
        {
            ServerTruthVersion = 99,
            OptimisticLocalKey = "idem-purge",
            PendingQueueKeys = new List<string> { "idem-pq-1", "idem-pq-2" },
            QueueHeld = true,
        };
        // 5 purge trigger を全て検証する
        var reasons = new[]
        {
            PurgeReason.Logout,
            PurgeReason.RefreshTokenExpiry,
            PurgeReason.TenantSwitch,
            PurgeReason.ActorSwitch,
            PurgeReason.DeviceBoundKeyRotate,
        };
        // 各 purge trigger で全 layer が初期化されることを確認する
        foreach (var reason in reasons)
        {
            var result = ClientStateReducer.ReducePurge(state, reason);
            // server_truth_version が null にリセットされることを確認する
            Assert.IsNull(result.NextState.ServerTruthVersion, $"[{reason}] ServerTruthVersion が null になること");
            // optimistic_local_key が null にリセットされることを確認する
            Assert.IsNull(result.NextState.OptimisticLocalKey, $"[{reason}] OptimisticLocalKey が null になること");
            // pending_queue_keys が空にリセットされることを確認する
            Assert.AreEqual(0, result.NextState.SafePendingQueueKeys.Count, $"[{reason}] PendingQueueKeys が空になること");
            // PurgeAllLayers action が含まれることを確認する
            Assert.IsTrue(
                result.Actions.OfType<PurgeAllLayersAction>().Any(a => a.Reason == reason),
                $"[{reason}] PurgeAllLayers({reason}) が含まれること");
        }
    }

    // ---------------------------------------------------------
    // TestPendingQueueResume_Empty_NoOp
    // ---------------------------------------------------------
    [TestMethod]
    public void TestPendingQueueResume_Empty_NoOp()
    {
        // PQ が空の state で pending_queue_resume を処理する
        var state = ClientState.Initial();
        // pending_queue_resume event を発行する
        var result = ClientStateReducer.Reduce(state, new PendingQueueResumeEvent());
        // PQ が空の場合は actions が空であることを確認する
        Assert.AreEqual(0, result.Actions.Count, "PQ 空 + resume = actions 空になること");
    }

    // ---------------------------------------------------------
    // TestPendingQueueResume_WithQueue_SendsInOrder
    // ---------------------------------------------------------
    [TestMethod]
    public void TestPendingQueueResume_WithQueue_SendsInOrder()
    {
        // PQ に entry がある state で pending_queue_resume を処理する
        var state = ClientState.Initial() with
        {
            PendingQueueKeys = new List<string> { "idem-a", "idem-b" },
        };
        // pending_queue_resume event を発行する
        var result = ClientStateReducer.Reduce(state, new PendingQueueResumeEvent());
        // SendQueueInOrder action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<SendQueueInOrderAction>().Any(),
            "PQ あり + resume = SendQueueInOrder が含まれること");
    }

    // ---------------------------------------------------------
    // TestBusinessConflict_Supersede_SilentToast
    // ---------------------------------------------------------
    [TestMethod]
    public void TestBusinessConflict_Supersede_SilentToast()
    {
        // PQ に entry がある state で supersede を受け取る
        var state = ClientState.Initial() with
        {
            PendingQueueKeys = new List<string> { "idem-supersede", "idem-next" },
        };
        // BusinessConflictReceived(Supersede) event を発行する
        var result = ClientStateReducer.Reduce(
            state,
            new BusinessConflictReceivedEvent(BusinessConflictSubtype.Supersede, "agg-sp-001"));
        // PQ 先頭 entry が削除されることを確認する
        Assert.IsFalse(
            result.NextState.SafePendingQueueKeys.Contains("idem-supersede"),
            "PQ 先頭 entry が削除されること");
        // 後続の PQ entry は残ることを確認する
        Assert.IsTrue(
            result.NextState.SafePendingQueueKeys.Contains("idem-next"),
            "後続 PQ entry は残ること");
        // NotifySilentToast action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<NotifySilentToastAction>().Any(),
            "NotifySilentToast が含まれること");
    }

    // ---------------------------------------------------------
    // TestBusinessConflict_ConcurrentEdit_Presence
    // ---------------------------------------------------------
    [TestMethod]
    public void TestBusinessConflict_ConcurrentEdit_Presence()
    {
        // 初期 state で concurrent_edit を受け取る
        var state = ClientState.Initial();
        // BusinessConflictReceived(ConcurrentEdit) event を発行する
        var result = ClientStateReducer.Reduce(
            state,
            new BusinessConflictReceivedEvent(BusinessConflictSubtype.ConcurrentEdit, "agg-ce-001"));
        // UpdatePresence action が含まれることを確認する
        Assert.IsTrue(
            result.Actions.OfType<UpdatePresenceAction>().Any(),
            "UpdatePresence が含まれること");
    }
}
