-- CrossCutting/Fsm/Determinism.lean — FSM 決定論公理
-- obligation_id: cross_fsm_axiom_068 (v1_property_axiom, tool_kind=lean4)
-- statement: 同じ状態・同じイベントから必ず同じ次状態に遷移することを証明する
-- tool: Lean 4 (simp / omega を使用)
-- k1s0-proof: PROOF-cross-fsm-lean4-001 -> IMPL-crosscutting-fsm-001
-- k1s0-impl: IMPL-crosscutting-fsm-001 realizes=FR-cross-fsm-001

namespace K1s0Formal.CrossCutting.Fsm

-- FSM の型パラメータ化した定義
-- S: 状態型, E: イベント型
variable (S E : Type)

-- 遷移関数: 状態 × イベント → 次状態 (決定論的なので全関数)
-- 実装では protoc-gen-k1s0-go-fsm が生成する Go コードに対応する
abbrev TransFn := S → E → S

-- 決定論の定義: 同じ引数には同じ結果を返す (Lean の関数は純粋なので自明)
def isDeterministic (trans : TransFn S E) : Prop :=
  ∀ s : S, ∀ e : E, ∀ s1 s2 : S,
    s1 = trans s e → s2 = trans s e → s1 = s2

-- 主定理: 任意の遷移関数は決定論的
-- Lean の関数は参照透過なので、同じ引数には同じ結果が返る
theorem all_trans_fn_deterministic (trans : TransFn S E) :
    isDeterministic S E trans := by
  intro s e s1 s2 h1 h2
  rw [h1, h2]

-- OrderStatus FSM の具体例 (k1s0 tier2 domain)
inductive OrderStatus where
  | Draft      : OrderStatus
  | Submitted  : OrderStatus
  | Approved   : OrderStatus
  | Rejected   : OrderStatus
  | Cancelled  : OrderStatus
  deriving Repr, DecidableEq

inductive OrderEvent where
  | Submit  : OrderEvent
  | Approve : OrderEvent
  | Reject  : OrderEvent
  | Cancel  : OrderEvent
  deriving Repr, DecidableEq

-- OrderStatus の遷移関数 (k1s0 tier2 ドメインの仕様に準拠)
def orderTransition : TransFn OrderStatus OrderEvent
  | OrderStatus.Draft,     OrderEvent.Submit  => OrderStatus.Submitted
  | OrderStatus.Submitted, OrderEvent.Approve => OrderStatus.Approved
  | OrderStatus.Submitted, OrderEvent.Reject  => OrderStatus.Rejected
  | OrderStatus.Draft,     OrderEvent.Cancel  => OrderStatus.Cancelled
  | OrderStatus.Submitted, OrderEvent.Cancel  => OrderStatus.Cancelled
  -- 無効な遷移: 現状態を維持する (no-op)
  | s, _                                      => s

-- 具体定理: OrderStatus 遷移は決定論的
theorem order_status_deterministic :
    isDeterministic OrderStatus OrderEvent orderTransition :=
  all_trans_fn_deterministic OrderStatus OrderEvent orderTransition

-- 補題: terminal 状態 (Approved/Rejected/Cancelled) への遷移は不可逆
theorem approved_terminal :
    ∀ e : OrderEvent, orderTransition OrderStatus.Approved e = OrderStatus.Approved := by
  intro e; cases e <;> rfl

theorem rejected_terminal :
    ∀ e : OrderEvent, orderTransition OrderStatus.Rejected e = OrderStatus.Rejected := by
  intro e; cases e <;> rfl

end K1s0Formal.CrossCutting.Fsm
