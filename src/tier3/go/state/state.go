// k1s0 tier3 4 layer client state reducer（Go 等価強度実装）
// TypeScript primary と同等の抽象を Go で実装する
// 適合仕様 11_クライアント状態適合仕様.md の v1 layer セットに準拠する
// Phase E: FieldDiff disjoint/intersect 分岐 + chainIdempotencyKey を追加する
package state

import (
	// crypto/rand パッケージ（暗号論的乱数生成）
	"crypto/rand"
	// encoding/hex パッケージ（バイト列を hex 文字列に変換）
	"encoding/hex"
	// fmt パッケージ（文字列フォーマット）
	"fmt"
	// strings パッケージ（文字列操作）
	"strings"
)

// FieldDiff は field-level diff を表す構造体（TypeScript FieldDiff と等価）
// ClientFields: client が変更したフィールド名のスライス
// ServerFields: server が変更したフィールド名のスライス
type FieldDiff struct {
	// client が変更したフィールド名のスライス
	ClientFields []string
	// server が変更したフィールド名のスライス
	ServerFields []string
}

// IsDisjoint は client / server の変更フィールドが disjoint かどうかを返す
// disjoint = client と server が同じフィールドを変更していない（rebase_clean）
func (fd *FieldDiff) IsDisjoint() bool {
	// clientFields を map に変換して O(1) 検索を可能にする
	clientSet := make(map[string]struct{}, len(fd.ClientFields))
	for _, f := range fd.ClientFields {
		// client フィールドを map に追加する
		clientSet[f] = struct{}{}
	}
	// serverFields に client との交差があるか確認する
	for _, f := range fd.ServerFields {
		// server フィールドが client フィールドに存在する場合は disjoint でない
		if _, ok := clientSet[f]; ok {
			return false
		}
	}
	// 交差がなければ disjoint（rebase_clean）
	return true
}

// Intersect は client / server の変更フィールドの交差（共通部分）を返す
// 交差が空でない場合は rebase_dirty → 3way merge UI が必要
func (fd *FieldDiff) Intersect() []string {
	// clientFields を map に変換して O(1) 検索を可能にする
	clientSet := make(map[string]struct{}, len(fd.ClientFields))
	for _, f := range fd.ClientFields {
		// client フィールドを map に追加する
		clientSet[f] = struct{}{}
	}
	// 交差フィールドを収集する
	var intersection []string
	for _, f := range fd.ServerFields {
		// server フィールドが client フィールドに存在する場合は交差として追加する
		if _, ok := clientSet[f]; ok {
			intersection = append(intersection, f)
		}
	}
	// 交差フィールドのスライスを返す（空スライスは nil になる可能性があるが問題ない）
	return intersection
}

// chainIdempotencyKey は base key から chain された新しい idempotency key を生成する
// TypeScript の chainIdempotencyKey（outbox.ts）と等価の実装
// base: 元の idempotency key（chain 親）
// next: 追加の識別子（aggregate ID + method のハッシュ等）
// wall-clock TTL 禁止規約に従い time.Now() を使用しない（crypto/rand のみ使用する）
func chainIdempotencyKey(base, next string) string {
	// 暗号論的乱数バイト列を 16 バイト生成する（UUID v4 相当の一意性を確保する）
	randBytes := make([]byte, 16)
	// crypto/rand で乱数を生成する（wall-clock 非依存）
	if _, err := rand.Read(randBytes); err != nil {
		// 乱数生成に失敗した場合は panic する（起動時の致命的エラー）
		panic(fmt.Sprintf("chainIdempotencyKey: crypto/rand.Read failed: %v", err))
	}
	// バイト列を hex 文字列に変換する
	randHex := hex.EncodeToString(randBytes)
	// next の先頭 8 文字を prefix に使用する（長すぎる場合は切り詰める）
	nextPrefix := next
	if len(nextPrefix) > 8 {
		nextPrefix = nextPrefix[:8]
	}
	// base の先頭 12 文字を prefix に使用する（長すぎる場合は切り詰める）
	basePrefix := base
	if len(basePrefix) > 12 {
		basePrefix = basePrefix[:12]
	}
	// chain された key を生成する（base_prefix + next_prefix + rand_hex）
	return strings.Join([]string{basePrefix, nextPrefix, randHex}, "_")
}

// LayerID は layer を識別する型
type LayerID string

const (
	// ServerTruth は tier2 atomic 三表書込確定値のキャッシュを表す
	ServerTruth LayerID = "server_truth"
	// OptimisticLocal は mutation in-flight overlay を表す
	OptimisticLocal LayerID = "optimistic_local"
	// PendingQueue は offline 永続化 mutation 経路を表す
	PendingQueue LayerID = "pending_queue"
	// Draft は編集中フォームの dirty state を表す
	Draft LayerID = "draft"
)

// BusinessConflictSubtype は BusinessConflict の subtype 型
type BusinessConflictSubtype string

const (
	// StaleWrite は field disjoint: rebase → auto resend を表す
	StaleWrite BusinessConflictSubtype = "stale_write"
	// LostUpdate は field intersect: 3way merge UI を表す
	LostUpdate BusinessConflictSubtype = "lost_update"
	// Supersede は同 actor 後続 op で既に更新済みを表す
	Supersede BusinessConflictSubtype = "supersede"
	// ConcurrentEdit は他 actor presence 中を表す
	ConcurrentEdit BusinessConflictSubtype = "concurrent_edit"
)

// PurgeReason は purge trigger の種別を表す型
type PurgeReason string

const (
	// Logout はログアウトによる purge を表す
	Logout PurgeReason = "logout"
	// RefreshTokenExpiry は refresh token 失効による purge を表す
	RefreshTokenExpiry PurgeReason = "refresh_token_expiry"
	// TenantSwitch はテナント切り替えによる purge を表す
	TenantSwitch PurgeReason = "tenant_switch"
	// ActorSwitch は actor 切り替えによる purge を表す
	ActorSwitch PurgeReason = "actor_switch"
	// DeviceBoundKeyRotate は device_bound_key ローテーションによる purge を表す
	DeviceBoundKeyRotate PurgeReason = "device_bound_key_rotate"
)

// ConflictEventType は conflict event の種別型
type ConflictEventType string

const (
	// EventServerTruthAdvance は server_truth バージョン増加 event を表す
	EventServerTruthAdvance ConflictEventType = "server_truth_advance"
	// EventOptimisticAcknowledged は mutation in-flight の ack event を表す
	EventOptimisticAcknowledged ConflictEventType = "optimistic_acknowledged"
	// EventOptimisticRejected は mutation in-flight の reject event を表す
	EventOptimisticRejected ConflictEventType = "optimistic_rejected"
	// EventPendingQueueResume はネットワーク復帰 / アプリ再開 event を表す
	EventPendingQueueResume ConflictEventType = "pending_queue_resume"
	// EventBusinessConflictReceived は api_response_409 + subtype event を表す
	EventBusinessConflictReceived ConflictEventType = "business_conflict_received"
)

// ConflictEvent は 5 conflict event のいずれかを表す
type ConflictEvent struct {
	// event の種別
	Type ConflictEventType
	// server_truth_advance: 新しい aggregate バージョン
	NewVersion int64
	// optimistic_acknowledged / rejected: idempotency_key
	IdempotencyKey string
	// optimistic_rejected: error code
	ErrorCode string
	// business_conflict_received: subtype
	Subtype BusinessConflictSubtype
	// business_conflict_received: field-level diff（stale_write / lost_update 判定に使用）
	// nil の場合は safe 側（dirty）にフォールバックする
	FieldDiff *FieldDiff
}

// ReducerActionType は reducer action の種別型
type ReducerActionType string

const (
	// ActionUpdateServerTruth は server_truth を更新する action を表す
	ActionUpdateServerTruth ReducerActionType = "update_server_truth"
	// ActionRollbackOptimistic は optimistic local を rollback する action を表す
	ActionRollbackOptimistic ReducerActionType = "rollback_optimistic"
	// ActionReEvaluatePendingQueue は pending queue を再評価する action を表す
	ActionReEvaluatePendingQueue ReducerActionType = "re_evaluate_pending_queue"
	// ActionPromoteOptimistic は optimistic を server_truth に promote する action を表す
	ActionPromoteOptimistic ReducerActionType = "promote_optimistic"
	// ActionDeletePqEntry は pending queue entry を削除する action を表す
	ActionDeletePqEntry ReducerActionType = "delete_pq_entry"
	// ActionSendQueueInOrder は queue を in-order で送信する action を表す
	ActionSendQueueInOrder ReducerActionType = "send_queue_in_order"
	// ActionPresentBusinessError は business error を表示する action を表す
	ActionPresentBusinessError ReducerActionType = "present_business_error"
	// ActionPresent3WayMergeUi は 3way merge UI を表示する action を表す
	ActionPresent3WayMergeUi ReducerActionType = "present_3way_merge_ui"
	// ActionNotifySilentToast は silent toast を表示する action を表す
	ActionNotifySilentToast ReducerActionType = "notify_silent_toast"
	// ActionHoldQueue は queue を hold する action を表す
	ActionHoldQueue ReducerActionType = "hold_queue"
	// ActionUpdatePresence は presence indicator を更新する action を表す
	ActionUpdatePresence ReducerActionType = "update_presence"
	// ActionPurgeAllLayers は全 layer を purge する action を表す
	ActionPurgeAllLayers ReducerActionType = "purge_all_layers"
)

// ReducerAction は reducer が返す副作用 action を表す
type ReducerAction struct {
	// action の種別
	Type ReducerActionType
	// detail（種別ごとに異なる）
	Detail any
}

// ClientState は 4 layer client state を表す
type ClientState struct {
	// server_truth の aggregate バージョン（0 = 未初期化）
	ServerTruthVersion int64
	// server_truth が初期化済みかどうか
	ServerTruthInitialized bool
	// optimistic_local の idempotency_key（空文字 = in-flight なし）
	OptimisticLocalKey string
	// pending_queue の idempotency_key リスト（enqueue 順）
	PendingQueueKeys []string
	// queue hold 中フラグ
	QueueHeld bool
}

// NewClientState は初期 state を生成する
func NewClientState() ClientState {
	// 全 layer を null / 空で初期化する
	return ClientState{
		PendingQueueKeys: []string{},
	}
}

// ReducerResult は reducer の結果を表す
type ReducerResult struct {
	// 次の client state
	NextState ClientState
	// 実行すべき副作用 actions
	Actions []ReducerAction
}

// ReducePurge は全 layer purge（5 trigger）を処理する
func ReducePurge(state ClientState, reason PurgeReason) ReducerResult {
	// 全 layer を purge して初期 state に戻す
	_ = state
	return ReducerResult{
		NextState: NewClientState(),
		Actions:   []ReducerAction{{Type: ActionPurgeAllLayers, Detail: reason}},
	}
}

// Reduce は 5 event を処理する決定論的 reducer
func Reduce(state ClientState, event ConflictEvent) ReducerResult {
	// event 種別に応じて reducer を分岐する
	switch event.Type {
	case EventServerTruthAdvance:
		// server_truth_advance: ST 更新 + OL rollback + PQ 再評価
		return reduceServerTruthAdvance(state, event.NewVersion)
	case EventOptimisticAcknowledged:
		// optimistic_acknowledged: OL → ST promote + PQ entry 削除
		return reduceOptimisticAcknowledged(state, event.IdempotencyKey)
	case EventOptimisticRejected:
		// optimistic_rejected: OL rollback + business error 表示
		return reduceOptimisticRejected(state, event.ErrorCode)
	case EventPendingQueueResume:
		// pending_queue_resume: PQ を in-order で送信する
		return reducePendingQueueResume(state)
	case EventBusinessConflictReceived:
		// business_conflict_received: subtype に応じて決定論的に dispatch する（FieldDiff があれば disjoint/intersect 分岐）
		return reduceBusinessConflictWithFieldDiff(state, event.Subtype, event.FieldDiff)
	default:
		// 未知の event は panic する
		panic("unknown conflict event type: " + string(event.Type))
	}
}

// reduceServerTruthAdvance は server_truth_advance event を処理する
func reduceServerTruthAdvance(state ClientState, newVersion int64) ReducerResult {
	// OL が存在する場合は rollback する
	actions := []ReducerAction{{Type: ActionUpdateServerTruth, Detail: newVersion}}
	nextState := state
	nextState.ServerTruthVersion = newVersion
	nextState.ServerTruthInitialized = true
	if state.OptimisticLocalKey != "" {
		nextState.OptimisticLocalKey = ""
		actions = append(actions, ReducerAction{Type: ActionRollbackOptimistic})
	}
	// PQ 再評価
	actions = append(actions, ReducerAction{Type: ActionReEvaluatePendingQueue})
	return ReducerResult{NextState: nextState, Actions: actions}
}

// reduceOptimisticAcknowledged は optimistic_acknowledged event を処理する
func reduceOptimisticAcknowledged(state ClientState, idempotencyKey string) ReducerResult {
	// OL を null にして PQ から entry を削除する
	nextState := state
	nextState.OptimisticLocalKey = ""
	filtered := []string{}
	for _, k := range state.PendingQueueKeys {
		if k != idempotencyKey {
			filtered = append(filtered, k)
		}
	}
	nextState.PendingQueueKeys = filtered
	return ReducerResult{
		NextState: nextState,
		Actions: []ReducerAction{
			{Type: ActionPromoteOptimistic, Detail: idempotencyKey},
			{Type: ActionDeletePqEntry, Detail: idempotencyKey},
		},
	}
}

// reduceOptimisticRejected は optimistic_rejected event を処理する
func reduceOptimisticRejected(state ClientState, errorCode string) ReducerResult {
	// OL を rollback する
	nextState := state
	nextState.OptimisticLocalKey = ""
	return ReducerResult{
		NextState: nextState,
		Actions: []ReducerAction{
			{Type: ActionRollbackOptimistic},
			{Type: ActionPresentBusinessError, Detail: errorCode},
		},
	}
}

// reducePendingQueueResume は pending_queue_resume event を処理する
func reducePendingQueueResume(state ClientState) ReducerResult {
	// PQ が空 / hold 中の場合は何もしない
	if state.QueueHeld || len(state.PendingQueueKeys) == 0 {
		return ReducerResult{NextState: state, Actions: []ReducerAction{}}
	}
	return ReducerResult{
		NextState: state,
		Actions:   []ReducerAction{{Type: ActionSendQueueInOrder}},
	}
}

// reduceBusinessConflictWithFieldDiff は field diff 付きの business conflict を処理する
// TypeScript subtypes.ts の resolveSubtypeActions と等価の FieldDiff disjoint/intersect 分岐を実装する
func reduceBusinessConflictWithFieldDiff(state ClientState, subtype BusinessConflictSubtype, fieldDiff *FieldDiff) ReducerResult {
	// FieldDiff がない場合は通常の reduceBusinessConflict にフォールバックする
	if fieldDiff == nil {
		return reduceBusinessConflict(state, subtype)
	}
	// stale_write の場合は FieldDiff disjoint/intersect で分岐する
	if subtype == StaleWrite {
		// FieldDiff.IsDisjoint() が true の場合は rebase_clean → auto resend
		if fieldDiff.IsDisjoint() {
			// OL の idempotency key を取得する（chain 元として使用する）
			baseKey := state.OptimisticLocalKey
			if baseKey == "" {
				// OL がない場合は safe 側（dirty）に倒す
				baseKey = "unknown"
			}
			// chain された新しい idempotency key を生成する
			newKey := chainIdempotencyKey(baseKey, "rebase")
			// rebase_clean: PQ の key を更新して auto resend する
			nextState := state
			nextState.OptimisticLocalKey = ""
			return ReducerResult{
				NextState: nextState,
				Actions: []ReducerAction{
					{Type: ActionRollbackOptimistic},
					{Type: ActionSendQueueInOrder, Detail: newKey},
				},
			}
		}
		// disjoint でない（intersect あり）: rebase_dirty → 3way merge UI + hold
		nextState := state
		nextState.QueueHeld = true
		return ReducerResult{
			NextState: nextState,
			Actions:   []ReducerAction{{Type: ActionPresent3WayMergeUi}, {Type: ActionHoldQueue}},
		}
	}
	// stale_write 以外は通常の reduceBusinessConflict にフォールバックする
	return reduceBusinessConflict(state, subtype)
}

// reduceBusinessConflict は business_conflict_received event を処理する
func reduceBusinessConflict(state ClientState, subtype BusinessConflictSubtype) ReducerResult {
	// subtype に応じて決定論的に actions を返す（分岐 override 禁止）
	switch subtype {
	case StaleWrite, LostUpdate:
		// safe 側（dirty）に倒して 3way merge UI + hold
		nextState := state
		nextState.QueueHeld = true
		return ReducerResult{
			NextState: nextState,
			Actions:   []ReducerAction{{Type: ActionPresent3WayMergeUi}, {Type: ActionHoldQueue}},
		}
	case Supersede:
		// PQ 先頭 entry 削除 + silent toast
		nextState := state
		actions := []ReducerAction{}
		if len(state.PendingQueueKeys) > 0 {
			key := state.PendingQueueKeys[0]
			nextState.PendingQueueKeys = state.PendingQueueKeys[1:]
			actions = append(actions, ReducerAction{Type: ActionDeletePqEntry, Detail: key})
		}
		actions = append(actions, ReducerAction{Type: ActionNotifySilentToast, Detail: "後続の操作で既に上書きされました"})
		return ReducerResult{NextState: nextState, Actions: actions}
	case ConcurrentEdit:
		// presence indicator 更新
		return ReducerResult{
			NextState: state,
			Actions:   []ReducerAction{{Type: ActionUpdatePresence, Detail: "unknown"}},
		}
	default:
		// 未知の subtype は panic する
		panic("unknown business conflict subtype: " + string(subtype))
	}
}
