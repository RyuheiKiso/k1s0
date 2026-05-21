// k1s0 tier3 4 layer client state reducer（Rust 等価強度実装）
// TypeScript primary と同等の抽象を Rust で実装する
// 適合仕様 11_クライアント状態適合仕様.md の v1 layer セットに準拠する
// Phase E: FieldDiff disjoint/intersect 分岐 + chain_idempotency_key を追加する

use serde::{Deserialize, Serialize};
// UUID 生成（idempotency key のランダム部分）
use uuid::Uuid;

// FieldDiff は field-level diff を表す構造体（TypeScript FieldDiff と等価）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDiff {
    // client が変更したフィールド名のリスト
    pub client_fields: Vec<String>,
    // server が変更したフィールド名のリスト
    pub server_fields: Vec<String>,
}

impl FieldDiff {
    // is_disjoint: client と server の変更フィールドが disjoint かどうかを返す
    // true = rebase_clean（auto resend 可能）
    // false = rebase_dirty（3way merge UI 必要）
    pub fn is_disjoint(&self) -> bool {
        // client_fields を HashSet に変換して O(1) 検索を可能にする
        let client_set: std::collections::HashSet<&str> =
            self.client_fields.iter().map(|s| s.as_str()).collect();
        // server_fields に client との交差があるか確認する
        !self.server_fields.iter().any(|f| client_set.contains(f.as_str()))
    }

    // intersect: client と server の変更フィールドの交差（共通部分）を返す
    pub fn intersect(&self) -> Vec<String> {
        // client_fields を HashSet に変換する
        let client_set: std::collections::HashSet<&str> =
            self.client_fields.iter().map(|s| s.as_str()).collect();
        // 交差フィールドを収集して返す
        self.server_fields
            .iter()
            .filter(|f| client_set.contains(f.as_str()))
            .cloned()
            .collect()
    }
}

// chain_idempotency_key: base key から chain された新しい idempotency key を生成する
// TypeScript の chainIdempotencyKey（outbox.ts）と等価の実装
// base: 元の idempotency key（chain 親）
// next: 追加の識別子（aggregate ID + method のハッシュ等）
pub fn chain_idempotency_key(base: &str, next: &str) -> String {
    // UUID v4 でランダムサフィックスを生成する
    let random_suffix = Uuid::new_v4().to_string().replace('-', "");
    // base の先頭 12 文字を prefix に使用する（長すぎる場合は切り詰める）
    let base_prefix = &base[..base.len().min(12)];
    // next の先頭 8 文字を prefix に使用する
    let next_prefix = &next[..next.len().min(8)];
    // chain された key を生成する（base_prefix + next_prefix + uuid_suffix）
    format!("{}_{}_{}", base_prefix, next_prefix, &random_suffix[..16])
}

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
        // field-level diff（stale_write / lost_update 判定に使用、None は safe 側にフォールバック）
        field_diff: Option<FieldDiff>,
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

// AutoResendWithChainedKeyAction は auto_resend_with_chained_key の型付きアクション
// T3-4: string メッセージではなく明示的な型付き enum variant で表現する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResendWithChainedKeyAction {
    // chain 元の idempotency_key（rebase 前の key）
    pub chained_from: String,
    // chain 後の新しい idempotency_key
    pub new_key: String,
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
    // BusinessConflict subtype action を dispatch する（auto_resend_with_chained_key 以外）
    DispatchConflictSubtype { subtype: BusinessConflictSubtype },
    // auto_resend_with_chained_key の型付きアクション（T3-4: string ではなく明示的型で表現する）
    AutoResendWithChainedKey { action: AutoResendWithChainedKeyAction },
    // 3way merge UI を表示する
    Present3WayMergeUi,
    // silent toast を表示する
    NotifySilentToast { message: String },
    // queue を hold する
    HoldQueue,
    // presence indicator を更新する
    UpdatePresence { actor_id: String },
    // server_truth を再取得する（stale_write / lost_update 後の rebase 前に最新を取得する）
    // TypeScript / Go の refetch_server_truth 相当（多言語 parity 維持のため追加）
    RefetchServerTruth { aggregate_id: String },
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
        ConflictEvent::BusinessConflictReceived { subtype, aggregate_id: _, field_diff } => {
            // business_conflict_received: subtype に応じて決定論的に dispatch する（FieldDiff 分岐含む）
            let mut actions = vec![];
            let mut next_state = state.clone();
            match subtype {
                BusinessConflictSubtype::StaleWrite => {
                    // stale_write: FieldDiff.is_disjoint() で rebase_clean/dirty を判定する
                    let is_clean = field_diff.as_ref().map(|fd| fd.is_disjoint()).unwrap_or(false);
                    if is_clean {
                        // rebase_clean: OL rollback + chain した新 key で auto resend する
                        let base_key = next_state.optimistic_local_key.clone().unwrap_or_default();
                        // chain した新しい idempotency_key を生成する
                        let new_key = chain_idempotency_key(&base_key, "rebase");
                        // OL を rollback する
                        next_state.optimistic_local_key = None;
                        // rollback action を追加する
                        actions.push(ReducerAction::RollbackOptimistic);
                        // T3-4: auto_resend_with_chained_key は型付き AutoResendWithChainedKey action として dispatch する
                        actions.push(ReducerAction::AutoResendWithChainedKey {
                            // AutoResendWithChainedKeyAction を設定する
                            action: AutoResendWithChainedKeyAction {
                                // chain 元の idempotency_key を設定する
                                chained_from: base_key,
                                // chain 後の新しい idempotency_key を設定する
                                new_key: new_key.clone(),
                            },
                        });
                        // auto resend action を追加する
                        actions.push(ReducerAction::SendQueueInOrder);
                    } else {
                        // rebase_dirty: safe 側に倒して 3way merge UI を表示する
                        next_state.queue_held = true;
                        actions.push(ReducerAction::Present3WayMergeUi);
                        actions.push(ReducerAction::HoldQueue);
                    }
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
            // field_diff なしのテスト（safe 側フォールバック）
            field_diff: None,
        };
        let result = reduce(&state, &event);
        // PQ が空になることを確認する
        assert!(result.next_state.pending_queue_keys.is_empty());
        // NOTIFY_SILENT_TOAST が含まれることを確認する
        assert!(result.actions.iter().any(|a| matches!(a, ReducerAction::NotifySilentToast { .. })));
    }

    // ==========================================================
    // Phase O 追加: 4 言語等価強度検証（Rust 側）
    // ==========================================================

    #[test]
    // Phase O: server_truth_advance で OL rollback → next_state.optimistic_local_key が None になることを確認する
    // Rust reducer は action-based 設計: server_truth_version の更新は UpdateServerTruth action で委譲する
    fn test_phase_o_server_truth_advance_rollbacks_optimistic() {
        // OL が存在する state を用意する
        let mut state = ClientState::new();
        state.optimistic_local_key = Some("idem-phase-o".to_string());
        // server_truth_version を None に設定する（未初期化）
        state.server_truth_version = None;
        // version=42 の server_truth_advance event を発行する
        let event = ConflictEvent::ServerTruthAdvance { new_version: 42 };
        let result = reduce(&state, &event);
        // OL が None になることを確認する（rollback）
        assert!(result.next_state.optimistic_local_key.is_none(), "OL は rollback で None になること");
        // ROLLBACK_OPTIMISTIC action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::RollbackOptimistic)),
            "RollbackOptimistic が含まれること"
        );
        // UPDATE_SERVER_TRUTH(42) action が含まれることを確認する（state 更新は action で委譲する）
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::UpdateServerTruth { version: 42 })),
            "UpdateServerTruth(42) が含まれること"
        );
        // RE_EVALUATE_PENDING_QUEUE action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::ReEvaluatePendingQueue)),
            "ReEvaluatePendingQueue が含まれること"
        );
    }

    #[test]
    // Phase O: purge で全 layer が初期化されることを確認する
    fn test_phase_o_purge_clears_all_layers() {
        // 全 layer に値が入った state を用意する
        let mut state = ClientState::new();
        state.server_truth_version = Some(99);
        state.optimistic_local_key = Some("idem-purge-o".to_string());
        state.pending_queue_keys = vec!["idem-pq-1".to_string(), "idem-pq-2".to_string()];
        state.queue_held = true;
        // Logout purge で全 layer が初期化されることを確認する
        let result = reduce_purge(&state, PurgeReason::Logout);
        // server_truth_version が None にリセットされることを確認する
        assert!(result.next_state.server_truth_version.is_none(), "server_truth_version が None になること");
        // optimistic_local_key が None にリセットされることを確認する
        assert!(result.next_state.optimistic_local_key.is_none(), "optimistic_local_key が None になること");
        // pending_queue_keys が空にリセットされることを確認する
        assert!(result.next_state.pending_queue_keys.is_empty(), "pending_queue_keys が空になること");
        // queue_held が false にリセットされることを確認する
        assert!(!result.next_state.queue_held, "queue_held が false になること");
        // PURGE_ALL_LAYERS action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::PurgeAllLayers { reason: PurgeReason::Logout })),
            "PurgeAllLayers(Logout) が含まれること"
        );
    }

    #[test]
    // Phase O: lost_update で 3way merge UI + HoldQueue が返ることを確認する
    fn test_phase_o_lost_update_3way_merge() {
        // 初期 state で lost_update conflict を受け取る
        let state = ClientState::new();
        // business_conflict_received(lost_update) event を発行する
        let event = ConflictEvent::BusinessConflictReceived {
            subtype: BusinessConflictSubtype::LostUpdate,
            aggregate_id: "agg-lu-001".to_string(),
            // field_diff なしのテスト（safe 側フォールバック）
            field_diff: None,
        };
        let result = reduce(&state, &event);
        // QueueHeld が true になることを確認する
        assert!(result.next_state.queue_held, "QueueHeld が true になること");
        // Present3WayMergeUi action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::Present3WayMergeUi)),
            "Present3WayMergeUi が含まれること"
        );
        // HoldQueue action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::HoldQueue)),
            "HoldQueue が含まれること"
        );
    }

    #[test]
    // Phase O: pending_queue_resume で PQ ありの場合 SendQueueInOrder が返ることを確認する
    fn test_phase_o_pending_queue_resume_sends_in_order() {
        // PQ に entry がある state を用意する
        let mut state = ClientState::new();
        state.pending_queue_keys = vec!["idem-a".to_string(), "idem-b".to_string()];
        // pending_queue_resume event を発行する
        let event = ConflictEvent::PendingQueueResume;
        let result = reduce(&state, &event);
        // SendQueueInOrder action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::SendQueueInOrder)),
            "SendQueueInOrder が含まれること"
        );
    }

    #[test]
    // Phase E: stale_write で FieldDiff.is_disjoint() が true の場合 rebase_clean になることを確認する
    fn test_phase_e_stale_write_rebase_clean_with_disjoint_field_diff() {
        // OL が存在する state で stale_write conflict を受け取る（disjoint field diff）
        let mut state = ClientState::new();
        state.optimistic_local_key = Some("idem-sw-clean".to_string());
        state.pending_queue_keys.push("idem-sw-clean".to_string());
        // client は qty を変更、server は price を変更（disjoint）
        let event = ConflictEvent::BusinessConflictReceived {
            subtype: BusinessConflictSubtype::StaleWrite,
            aggregate_id: "agg-sw-001".to_string(),
            // disjoint field diff を設定する（client=qty / server=price）
            field_diff: Some(FieldDiff {
                client_fields: vec!["qty".to_string()],
                server_fields: vec!["price".to_string()],
            }),
        };
        let result = reduce(&state, &event);
        // rebase_clean: QueueHeld が false のままであることを確認する
        assert!(!result.next_state.queue_held, "rebase_clean: QueueHeld が false のままであること");
        // RollbackOptimistic action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::RollbackOptimistic)),
            "RollbackOptimistic が含まれること"
        );
        // SendQueueInOrder action が含まれることを確認する（auto resend）
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::SendQueueInOrder)),
            "SendQueueInOrder が含まれること（auto resend）"
        );
    }

    #[test]
    // Phase E: FieldDiff.is_disjoint() が false の場合 rebase_dirty になることを確認する
    fn test_phase_e_stale_write_rebase_dirty_with_intersecting_field_diff() {
        // OL が存在する state で stale_write conflict を受け取る（intersecting field diff）
        let state = ClientState::new();
        // client と server が両方 qty を変更（intersecting）
        let event = ConflictEvent::BusinessConflictReceived {
            subtype: BusinessConflictSubtype::StaleWrite,
            aggregate_id: "agg-sw-002".to_string(),
            // intersecting field diff を設定する（client=qty,price / server=qty）
            field_diff: Some(FieldDiff {
                client_fields: vec!["qty".to_string(), "price".to_string()],
                server_fields: vec!["qty".to_string()],
            }),
        };
        let result = reduce(&state, &event);
        // rebase_dirty: QueueHeld が true になることを確認する
        assert!(result.next_state.queue_held, "rebase_dirty: QueueHeld が true になること");
        // Present3WayMergeUi action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::Present3WayMergeUi)),
            "Present3WayMergeUi が含まれること"
        );
    }

    #[test]
    // Phase E: chain_idempotency_key が base と next を含む新しい key を生成することを確認する
    fn test_phase_e_chain_idempotency_key() {
        // chain_idempotency_key で新しい key が生成されることを確認する
        let base = "idem-base-001";
        let next = "rebase";
        // chain された key を生成する
        let chained = chain_idempotency_key(base, next);
        // chained key が空でないことを確認する
        assert!(!chained.is_empty(), "chain された key が空でないこと");
        // chained key が base の先頭 12 文字を含むことを確認する
        assert!(chained.starts_with(&base[..base.len().min(12)]), "chained key が base prefix を含むこと");
        // chained key が base と異なることを確認する（新しい key が生成された）
        assert_ne!(chained, base, "chained key が base と異なること");
    }

    #[test]
    // Phase O: concurrent_edit で UpdatePresence action が返ることを確認する
    fn test_phase_o_concurrent_edit_updates_presence() {
        // 初期 state で concurrent_edit を受け取る
        let state = ClientState::new();
        // business_conflict_received(concurrent_edit) event を発行する
        let event = ConflictEvent::BusinessConflictReceived {
            subtype: BusinessConflictSubtype::ConcurrentEdit,
            aggregate_id: "agg-ce-001".to_string(),
            // field_diff なしのテスト（safe 側フォールバック）
            field_diff: None,
        };
        let result = reduce(&state, &event);
        // UpdatePresence action が含まれることを確認する
        assert!(
            result.actions.iter().any(|a| matches!(a, ReducerAction::UpdatePresence { .. })),
            "UpdatePresence が含まれること"
        );
    }
}
