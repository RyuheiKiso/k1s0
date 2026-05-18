// k1s0 tier3 Go state reducer テスト（4 言語等価強度検証）
// TypeScript primary と同等の決定論的 reducer 挙動を Go で検証する
// 適合仕様 11_クライアント状態適合仕様.md の v1 layer セットに準拠する
package state

// testing パッケージをインポートする
import "testing"

// ---------------------------------------------------------
// TestServerTruthAdvance
// ---------------------------------------------------------

// TestServerTruthAdvance は OL なし時に UPDATE_SERVER_TRUTH + RE_EVALUATE が返ることを確認する
func TestServerTruthAdvance(t *testing.T) {
	// 初期 state を生成する
	state := NewClientState()
	// version=7 の server_truth_advance event を発行する
	ev := ConflictEvent{Type: EventServerTruthAdvance, NewVersion: 7}
	result := Reduce(state, ev)
	// server_truth_version が 7 に更新されることを確認する
	if result.NextState.ServerTruthVersion != 7 {
		t.Errorf("ServerTruthVersion: got %d, want 7", result.NextState.ServerTruthVersion)
	}
	// UPDATE_SERVER_TRUTH action が含まれることを確認する
	if !containsAction(result.Actions, ActionUpdateServerTruth) {
		t.Error("ActionUpdateServerTruth が含まれること")
	}
	// RE_EVALUATE_PENDING_QUEUE action が含まれることを確認する
	if !containsAction(result.Actions, ActionReEvaluatePendingQueue) {
		t.Error("ActionReEvaluatePendingQueue が含まれること")
	}
}

// TestServerTruthAdvance_RollbacksOptimistic は OL ありの state で ROLLBACK_OPTIMISTIC が返ることを確認する
func TestServerTruthAdvance_RollbacksOptimistic(t *testing.T) {
	// OL が存在する state を用意する
	state := NewClientState()
	state.OptimisticLocalKey = "idem-001"
	// version=3 の server_truth_advance event を発行する
	ev := ConflictEvent{Type: EventServerTruthAdvance, NewVersion: 3}
	result := Reduce(state, ev)
	// ROLLBACK_OPTIMISTIC action が含まれることを確認する
	if !containsAction(result.Actions, ActionRollbackOptimistic) {
		t.Error("ActionRollbackOptimistic が含まれること")
	}
	// next state で OL が空文字になることを確認する
	if result.NextState.OptimisticLocalKey != "" {
		t.Errorf("OptimisticLocalKey: got %q, want empty", result.NextState.OptimisticLocalKey)
	}
}

// ---------------------------------------------------------
// TestOptimisticAcknowledged
// ---------------------------------------------------------

// TestOptimisticAcknowledged は PQ entry が削除されて PROMOTE_OPTIMISTIC が返ることを確認する
func TestOptimisticAcknowledged(t *testing.T) {
	// PQ に "idem-ack" が存在する state を用意する
	state := NewClientState()
	state.OptimisticLocalKey = "idem-ack"
	state.PendingQueueKeys = []string{"idem-ack"}
	// optimistic_acknowledged event を発行する
	ev := ConflictEvent{Type: EventOptimisticAcknowledged, IdempotencyKey: "idem-ack"}
	result := Reduce(state, ev)
	// PQ が空になることを確認する
	if len(result.NextState.PendingQueueKeys) != 0 {
		t.Errorf("PendingQueueKeys: got %v, want empty", result.NextState.PendingQueueKeys)
	}
	// OL が空文字になることを確認する
	if result.NextState.OptimisticLocalKey != "" {
		t.Errorf("OptimisticLocalKey: got %q, want empty", result.NextState.OptimisticLocalKey)
	}
	// PROMOTE_OPTIMISTIC action が含まれることを確認する
	if !containsAction(result.Actions, ActionPromoteOptimistic) {
		t.Error("ActionPromoteOptimistic が含まれること")
	}
}

// ---------------------------------------------------------
// TestBusinessConflictStaleWrite
// ---------------------------------------------------------

// TestBusinessConflictStaleWrite は stale_write で 3way merge UI + HoldQueue が返ることを確認する
func TestBusinessConflictStaleWrite(t *testing.T) {
	// 初期 state で stale_write conflict を受け取る
	state := NewClientState()
	// business_conflict_received(stale_write) event を発行する
	ev := ConflictEvent{Type: EventBusinessConflictReceived, Subtype: StaleWrite}
	result := Reduce(state, ev)
	// QueueHeld が true になることを確認する
	if !result.NextState.QueueHeld {
		t.Error("QueueHeld が true になること")
	}
	// Present3WayMergeUi action が含まれることを確認する
	if !containsAction(result.Actions, ActionPresent3WayMergeUi) {
		t.Error("ActionPresent3WayMergeUi が含まれること")
	}
	// HoldQueue action が含まれることを確認する
	if !containsAction(result.Actions, ActionHoldQueue) {
		t.Error("ActionHoldQueue が含まれること")
	}
}

// TestBusinessConflict_LostUpdate は lost_update で 3way merge UI + HoldQueue が返ることを確認する
func TestBusinessConflict_LostUpdate(t *testing.T) {
	// 初期 state で lost_update conflict を受け取る
	state := NewClientState()
	// business_conflict_received(lost_update) event を発行する
	ev := ConflictEvent{Type: EventBusinessConflictReceived, Subtype: LostUpdate}
	result := Reduce(state, ev)
	// QueueHeld が true になることを確認する
	if !result.NextState.QueueHeld {
		t.Error("QueueHeld が true になること（lost_update は 3way merge UI）")
	}
	// Present3WayMergeUi action が含まれることを確認する
	if !containsAction(result.Actions, ActionPresent3WayMergeUi) {
		t.Error("ActionPresent3WayMergeUi が含まれること")
	}
}

// TestBusinessConflict_Supersede は supersede で PQ 先頭 entry 削除 + SilentToast が返ることを確認する
func TestBusinessConflict_Supersede(t *testing.T) {
	// PQ に entry がある state で supersede を受け取る
	state := NewClientState()
	state.PendingQueueKeys = []string{"idem-head", "idem-tail"}
	// business_conflict_received(supersede) event を発行する
	ev := ConflictEvent{Type: EventBusinessConflictReceived, Subtype: Supersede}
	result := Reduce(state, ev)
	// PQ 先頭が削除されることを確認する
	for _, k := range result.NextState.PendingQueueKeys {
		if k == "idem-head" {
			t.Error("PQ 先頭 entry が削除されること（idem-head は残ってはいけない）")
		}
	}
	// NotifySilentToast action が含まれることを確認する
	if !containsAction(result.Actions, ActionNotifySilentToast) {
		t.Error("ActionNotifySilentToast が含まれること")
	}
}

// TestBusinessConflict_ConcurrentEdit は concurrent_edit で UpdatePresence が返ることを確認する
func TestBusinessConflict_ConcurrentEdit(t *testing.T) {
	// 初期 state で concurrent_edit を受け取る
	state := NewClientState()
	// business_conflict_received(concurrent_edit) event を発行する
	ev := ConflictEvent{Type: EventBusinessConflictReceived, Subtype: ConcurrentEdit}
	result := Reduce(state, ev)
	// UpdatePresence action が含まれることを確認する
	if !containsAction(result.Actions, ActionUpdatePresence) {
		t.Error("ActionUpdatePresence が含まれること")
	}
}

// ---------------------------------------------------------
// TestPurgeAllLayers
// ---------------------------------------------------------

// TestPurgeAllLayers は 5 purge trigger で全 layer が初期化されることを確認する
func TestPurgeAllLayers(t *testing.T) {
	// 全 layer に値が入った state を用意する
	state := NewClientState()
	state.ServerTruthVersion = 99
	state.ServerTruthInitialized = true
	state.OptimisticLocalKey = "idem-purge"
	state.PendingQueueKeys = []string{"idem-pq-1", "idem-pq-2"}
	state.QueueHeld = true
	// 5 purge trigger を全て検証する
	reasons := []PurgeReason{
		Logout,
		RefreshTokenExpiry,
		TenantSwitch,
		ActorSwitch,
		DeviceBoundKeyRotate,
	}
	// 各 purge trigger で全 layer が初期化されることを確認する
	for _, reason := range reasons {
		result := ReducePurge(state, reason)
		// server_truth_initialized が false にリセットされることを確認する
		if result.NextState.ServerTruthInitialized {
			t.Errorf("[%s] ServerTruthInitialized が false になること", reason)
		}
		// optimistic_local_key が空文字にリセットされることを確認する
		if result.NextState.OptimisticLocalKey != "" {
			t.Errorf("[%s] OptimisticLocalKey が空文字になること", reason)
		}
		// pending_queue_keys が空にリセットされることを確認する
		if len(result.NextState.PendingQueueKeys) != 0 {
			t.Errorf("[%s] PendingQueueKeys が空になること", reason)
		}
		// PURGE_ALL_LAYERS action が含まれることを確認する
		if !containsAction(result.Actions, ActionPurgeAllLayers) {
			t.Errorf("[%s] ActionPurgeAllLayers が含まれること", reason)
		}
	}
}

// ---------------------------------------------------------
// TestPendingQueueResume
// ---------------------------------------------------------

// TestPendingQueueResume_EmptyQueue は PQ が空の場合 actions が空であることを確認する
func TestPendingQueueResume_EmptyQueue(t *testing.T) {
	// PQ が空の state で pending_queue_resume を処理する
	state := NewClientState()
	// pending_queue_resume event を発行する
	ev := ConflictEvent{Type: EventPendingQueueResume}
	result := Reduce(state, ev)
	// actions が空であることを確認する
	if len(result.Actions) != 0 {
		t.Errorf("PQ 空 + resume = actions 空になること: got %v", result.Actions)
	}
}

// TestPendingQueueResume_WithQueue は PQ あり時に SendQueueInOrder が返ることを確認する
func TestPendingQueueResume_WithQueue(t *testing.T) {
	// PQ に entry がある state で pending_queue_resume を処理する
	state := NewClientState()
	state.PendingQueueKeys = []string{"idem-a", "idem-b"}
	// pending_queue_resume event を発行する
	ev := ConflictEvent{Type: EventPendingQueueResume}
	result := Reduce(state, ev)
	// SendQueueInOrder action が含まれることを確認する
	if !containsAction(result.Actions, ActionSendQueueInOrder) {
		t.Error("ActionSendQueueInOrder が含まれること")
	}
}

// ---------------------------------------------------------
// ヘルパー関数
// ---------------------------------------------------------

// containsAction は actions スライスに指定した type の action が含まれるかを確認するヘルパー
func containsAction(actions []ReducerAction, actionType ReducerActionType) bool {
	// actions を線形探索して type が一致するものを探す
	for _, a := range actions {
		if a.Type == actionType {
			return true
		}
	}
	// 見つからない場合は false を返す
	return false
}
