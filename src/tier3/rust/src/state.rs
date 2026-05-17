// k1s0 tier3 4 layer client state reducer（Rust 等価強度実装）
// TypeScript primary と同等の抽象を Rust で実装する
// 適合仕様 11_クライアント状態適合仕様.md の v1 layer セットに準拠する

use serde::{Deserialize, Serialize};

// layer を識別する enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerId {
    // tier2 atomic 三表書込確定値のキャッシュ
    ServerTruth,
    // mutation in-flight overlay
    OptimisticLocal,
    // offline 永続化 mutation 経路
    PendingQueue,
    // 編集中フォームの dirty state
    Draft,
}

// 5 conflict event の enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictEvent {
    // server_truth バージョン増加
    ServerTruthAdvance { new_version: u64 },
    // mutation in-flight の ack
    OptimisticAcknowledged { idempotency_key: String },
    // mutation in-flight の reject
    OptimisticRejected {
        idempotency_key: String,
        error_code: String,
        conflict_subtype: Option<BusinessConflictSubtype>,
    },
    // ネットワーク復帰 / アプリ再開
    PendingQueueResume,
    // api_response_409 + subtype
    BusinessConflictReceived {
        subtype: BusinessConflictSubtype,
        aggregate_id: String,
    },
}

// BusinessConflict subtype（4 種固定）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusinessConflictSubtype {
    // field disjoint: rebase → auto resend
    StaleWrite,
    // field intersect: 3way merge UI
    LostUpdate,
    // 同 actor 後続 op で既に更新済み
    Supersede,
    // 他 actor presence 中
    ConcurrentEdit,
}

// 5 purge trigger enum（全 layer purge に使用）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurgeReason {
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

// reducer が返す副作用 actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReducerAction {
    // server_truth を更新する
    UpdateServerTruth { version: u64 },
    // optimistic local を rollback する
    RollbackOptimistic,
    // pending queue を再評価する
    ReEvaluatePendingQueue,
    // optimistic を server_truth に promote する
    PromoteOptimistic { idempotency_key: String },
    // pending queue entry を削除する
    DeletePqEntry { idempotency_key: String },
    // queue を in-order で送信する
    SendQueueInOrder,
    // business error を表示する
    PresentBusinessError { error_code: String },
    // BusinessConflict subtype action を dispatch する
    DispatchConflictSubtype { subtype: BusinessConflictSubtype },
    // 3way merge UI を表示する
    Present3WayMergeUi,
    // silent toast を表示する
    NotifySilentToast { message: String },
    // queue を hold する
    HoldQueue,
    // presence indicator を更新する
    UpdatePresence { actor_id: String },
    // 全 layer を purge する
    PurgeAllLayers { reason: PurgeReason },
}

// 4 layer client state（generics T は aggregate の値型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientState {
    // server_truth の aggregate バージョン（None = 未初期化）
    pub server_truth_version: Option<u64>,
    // optimistic_local の idempotency_key（None = in-flight なし）
    pub optimistic_local_key: Option<String>,
    // pending_queue の idempotency_key リスト（enqueue 順）
    pub pending_queue_keys: Vec<String>,
    // queue hold 中フラグ
    pub queue_held: bool,
}

impl ClientState {
    // 初期 state を生成する
    pub fn new() -> Self {
        // 全 layer を null / 空で初期化する
        Self {
            server_truth_version: None,
            optimistic_local_key: None,
            pending_queue_keys: Vec::new(),
            queue_held: false,
        }
    }
}

impl Default for ClientState {
    // Default を new() に委譲する
    fn default() -> Self {
        Self::new()
    }
}

// reducer の結果
pub struct ReducerResult {
    // 次の client state
    pub next_state: ClientState,
    // 実行すべき副作用 actions
    pub actions: Vec<ReducerAction>,
}

// 全 layer purge（5 trigger）
pub fn reduce_purge(state: &ClientState, reason: PurgeReason) -> ReducerResult {
    // 全 layer を purge して初期 state に戻す
    ReducerResult {
        next_state: ClientState::new(),
        actions: vec![ReducerAction::PurgeAllLayers { reason }],
    }
}

// 5 event を処理する決定論的 reducer
pub fn reduce(state: &ClientState, event: &ConflictEvent) -> ReducerResult {
    // event 種別に応じて reducer を分岐する
    match event {
        ConflictEvent::ServerTruthAdvance { new_version } => {
            // server_truth_advance: ST 更新 + OL rollback + PQ 再評価
            let mut actions = vec![ReducerAction::UpdateServerTruth { version: *new_version }];
            let mut next_state = state.clone();
            // OL が存在する場合は rollback する
            if state.optimistic_local_key.is_some() {
                next_state.optimistic_local_key = None;
                actions.push(ReducerAction::RollbackOptimistic);
            }
            // PQ 再評価
            actions.push(ReducerAction::ReEvaluatePendingQueue);
            ReducerResult { next_state, actions }
        }
        ConflictEvent::OptimisticAcknowledged { idempotency_key } => {
            // optimistic_acknowledged: OL → ST promote + PQ entry 削除
            let mut next_state = state.clone();
            next_state.optimistic_local_key = None;
            // 対応する PQ entry を削除する
            next_state.pending_queue_keys.retain(|k| k != idempotency_key);
            ReducerResult {
                next_state,
                actions: vec![
                    ReducerAction::PromoteOptimistic { idempotency_key: idempotency_key.clone() },
                    ReducerAction::DeletePqEntry { idempotency_key: idempotency_key.clone() },
                ],
            }
        }
        ConflictEvent::OptimisticRejected {
            idempotency_key: _,
            error_code,
            conflict_subtype,
        } => {
            // optimistic_rejected: OL rollback + business error 表示
            let mut next_state = state.clone();
            next_state.optimistic_local_key = None;
            let mut actions = vec![
                ReducerAction::RollbackOptimistic,
                ReducerAction::PresentBusinessError { error_code: error_code.clone() },
            ];
            // BusinessConflict の場合は subtype dispatch を追加する
            if let Some(subtype) = conflict_subtype {
                actions.push(ReducerAction::DispatchConflictSubtype { subtype: subtype.clone() });
            }
            ReducerResult { next_state, actions }
        }
        ConflictEvent::PendingQueueResume => {
            // pending_queue_resume: PQ を in-order で送信する
            if state.queue_held || state.pending_queue_keys.is_empty() {
                // hold 中 / 空 の場合は何もしない
                return ReducerResult { next_state: state.clone(), actions: vec![] };
            }
            ReducerResult {
                next_state: state.clone(),
                actions: vec![ReducerAction::SendQueueInOrder],
            }
        }
        ConflictEvent::BusinessConflictReceived { subtype, aggregate_id: _ } => {
            // business_conflict_received: subtype に応じて決定論的に dispatch する
            let mut actions = vec![];
            let mut next_state = state.clone();
            match subtype {
                BusinessConflictSubtype::StaleWrite => {
                    // stale_write: rebase → auto resend（clean）または 3way merge UI（dirty）
                    // ここでは safe 側（dirty）に倒して 3way merge UI を表示する
                    next_state.queue_held = true;
                    actions.push(ReducerAction::Present3WayMergeUi);
                    actions.push(ReducerAction::HoldQueue);
                }
                BusinessConflictSubtype::LostUpdate => {
                    // lost_update: 3way merge UI + queue hold
                    next_state.queue_held = true;
                    actions.push(ReducerAction::Present3WayMergeUi);
                    actions.push(ReducerAction::HoldQueue);
                }
                BusinessConflictSubtype::Supersede => {
                    // supersede: PQ の先頭 entry を削除 + silent toast
                    if let Some(key) = next_state.pending_queue_keys.first().cloned() {
                        next_state.pending_queue_keys.remove(0);
                        actions.push(ReducerAction::DeletePqEntry { idempotency_key: key });
                    }
                    actions.push(ReducerAction::NotifySilentToast {
                        message: "後続の操作で既に上書きされました".to_string(),
                    });
                }
                BusinessConflictSubtype::ConcurrentEdit => {
                    // concurrent_edit: presence indicator 更新 + user choice
                    actions.push(ReducerAction::UpdatePresence { actor_id: "unknown".to_string() });
                }
            }
            ReducerResult { next_state, actions }
        }
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // server_truth_advance event で UPDATE_SERVER_TRUTH action が返ることを確認する
    fn test_server_truth_advance_returns_update_action() {
        // 初期 state で server_truth_advance を処理する
        let state = ClientState::new();
        let event = ConflictEvent::ServerTruthAdvance { new_version: 2 };
        let result = reduce(&state, &event);
        // UPDATE_SERVER_TRUTH が含まれることを確認する
        assert!(result.actions.iter().any(|a| matches!(a, ReducerAction::UpdateServerTruth { .. })));
        // RE_EVALUATE_PENDING_QUEUE が含まれることを確認する
        assert!(result.actions.iter().any(|a| matches!(a, ReducerAction::ReEvaluatePendingQueue)));
    }

    #[test]
    // OL が存在するとき server_truth_advance で ROLLBACK_OPTIMISTIC action が返ることを確認する
    fn test_server_truth_advance_rollbacks_optimistic() {
        // OL ありの state で server_truth_advance を処理する
        let mut state = ClientState::new();
        state.optimistic_local_key = Some("idem-001".to_string());
        let event = ConflictEvent::ServerTruthAdvance { new_version: 3 };
        let result = reduce(&state, &event);
        // ROLLBACK_OPTIMISTIC が含まれることを確認する
        assert!(result.actions.iter().any(|a| matches!(a, ReducerAction::RollbackOptimistic)));
        // next state で OL が None になることを確認する
        assert!(result.next_state.optimistic_local_key.is_none());
    }

    #[test]
    // optimistic_acknowledged event で PQ entry が削除されることを確認する
    fn test_optimistic_acknowledged_deletes_pq_entry() {
        // PQ に entry がある state で optimistic_acknowledged を処理する
        let mut state = ClientState::new();
        state.pending_queue_keys.push("idem-002".to_string());
        let event = ConflictEvent::OptimisticAcknowledged {
            idempotency_key: "idem-002".to_string(),
        };
        let result = reduce(&state, &event);
        // PQ が空になることを確認する
        assert!(result.next_state.pending_queue_keys.is_empty());
        // PROMOTE_OPTIMISTIC が含まれることを確認する
        assert!(result.actions.iter().any(|a| matches!(a, ReducerAction::PromoteOptimistic { .. })));
    }

    #[test]
    // 5 purge trigger で全 layer が purge されることを確認する
    fn test_all_purge_triggers() {
        // 全 purge trigger で PurgeAllLayers action が返ることを確認する
        let purge_reasons = [
            PurgeReason::Logout,
            PurgeReason::RefreshTokenExpiry,
            PurgeReason::TenantSwitch,
            PurgeReason::ActorSwitch,
            PurgeReason::DeviceBoundKeyRotate,
        ];
        for reason in purge_reasons {
            let mut state = ClientState::new();
            state.pending_queue_keys.push("idem-purge".to_string());
            let result = reduce_purge(&state, reason);
            // PQ が空になることを確認する
            assert!(result.next_state.pending_queue_keys.is_empty());
            // PURGE_ALL_LAYERS が含まれることを確認する
            assert!(result.actions.iter().any(|a| matches!(a, ReducerAction::PurgeAllLayers { .. })));
        }
    }

    #[test]
    // supersede subtype で PQ entry が削除されることを確認する
    fn test_business_conflict_supersede_deletes_pq() {
        // PQ に entry がある state で supersede を処理する
        let mut state = ClientState::new();
        state.pending_queue_keys.push("idem-supersede".to_string());
        let event = ConflictEvent::BusinessConflictReceived {
            subtype: BusinessConflictSubtype::Supersede,
            aggregate_id: "agg-001".to_string(),
        };
        let result = reduce(&state, &event);
        // PQ が空になることを確認する
        assert!(result.next_state.pending_queue_keys.is_empty());
        // NOTIFY_SILENT_TOAST が含まれることを確認する
        assert!(result.actions.iter().any(|a| matches!(a, ReducerAction::NotifySilentToast { .. })));
    }
}
