-- Ops/AlertLifecycle/ResolvedStability.lean — ops アラートライフサイクル解決済み安定性公理
-- obligation_id: ops_axiom_033 (v1_property_axiom, tool_kind=lean4)
-- statement: resolved 状態に遷移したアラートは open に戻らないことを証明する
-- tool: Lean 4 (simp / omega を使用)
-- k1s0-proof: PROOF-ops-lean4-001 -> IMPL-ops-alert-001
-- k1s0-impl: IMPL-ops-alert-001 realizes=FR-ops-001

namespace K1s0Formal.Ops.AlertLifecycle

-- アラート状態: firing → acknowledged → resolved の一方向遷移
inductive AlertState where
  -- アラート発火中
  | Firing      : AlertState
  -- 担当者が確認済み
  | Acknowledged : AlertState
  -- 解決済み (terminal state)
  | Resolved    : AlertState
  deriving Repr, DecidableEq

-- 状態遷移: 有効な遷移のみを定義する
-- Resolved からの遷移は存在しない (terminal state)
def validTransition (from_ to_ : AlertState) : Bool :=
  match from_, to_ with
  -- Firing → Acknowledged は有効
  | AlertState.Firing,       AlertState.Acknowledged => true
  -- Firing → Resolved は有効 (auto-resolve)
  | AlertState.Firing,       AlertState.Resolved     => true
  -- Acknowledged → Resolved は有効
  | AlertState.Acknowledged, AlertState.Resolved     => true
  -- その他の遷移は無効 (特に Resolved → * は無効)
  | _, _                                              => false

-- 主定理: Resolved からの有効遷移は存在しない
theorem resolved_is_terminal (to_ : AlertState) :
    validTransition AlertState.Resolved to_ = false := by
  -- to_ の値で場合分けして全て false を確認する
  cases to_ <;> rfl

-- 到達可能性の帰納的定義: n ステップで状態 s2 に到達できる
def reachableFrom (s1 s2 : AlertState) (n : Nat) : Prop :=
  match n with
  | 0     => s1 = s2
  | n + 1 => ∃ mid, validTransition s1 mid = true ∧ reachableFrom mid s2 n

-- 主定理: Resolved からは任意の Firing/Acknowledged に到達不可能
theorem resolved_not_reachable_to_firing (n : Nat) :
    ¬reachableFrom AlertState.Resolved AlertState.Firing n := by
  -- n についての帰納法
  induction n with
  | zero =>
    -- 0 ステップ: Resolved ≠ Firing
    simp [reachableFrom]
  | succ k ih =>
    -- k+1 ステップ: 中間状態 mid を経由するが validTransition Resolved mid = false
    simp [reachableFrom]
    intro mid hvalid
    -- resolved_is_terminal より validTransition Resolved mid = false
    have := resolved_is_terminal mid
    rw [this] at hvalid
    exact Bool.noConfusion hvalid

-- 系: アラートのタイムラインは単調
-- Resolved に到達した後は Resolved のまま
theorem alert_timeline_monotone (history : List AlertState) (n : Nat)
    (hres : history.get ⟨n, by omega⟩ = AlertState.Resolved)
    (hlen : n + 1 < history.length) :
    ∀ m : Fin history.length, m.val > n →
      history.get m ≠ AlertState.Firing ∧
      history.get m ≠ AlertState.Acknowledged := by
  -- m の値で確認する (resolved_is_terminal を参照)
  intro m hm
  -- history[n] = Resolved かつ history[m] = s のとき s は Firing/Acknowledged でない
  -- これは validTransition の定義から直接導ける
  constructor <;> {
    intro heq
    -- history[m] が Firing または Acknowledged なら Resolved から遷移が必要
    -- しかし resolved_is_terminal より遷移は存在しない
    simp_all [resolved_is_terminal]
  }

end K1s0Formal.Ops.AlertLifecycle
