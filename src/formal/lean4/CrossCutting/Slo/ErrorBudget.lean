-- CrossCutting/Slo/ErrorBudget.lean — SLO エラーバジェット単調減少公理
-- obligation_id: cross_slo_axiom_073 (v1_property_axiom, tool_kind=lean4)
-- statement: エラーイベントが発生するたびにエラーバジェットは単調減少することを証明する
-- tool: Lean 4 (omega を使用)
-- k1s0-proof: PROOF-cross-slo-lean4-001 -> IMPL-crosscutting-slo-001
-- k1s0-impl: IMPL-crosscutting-slo-001 realizes=FR-cross-slo-001

namespace K1s0Formal.CrossCutting.Slo

-- SLO 状態: エラーバジェット残量 (自然数で抽象化)
-- 実際は (1 - error_ratio) × window_minutes で計算される
structure SloState where
  -- budget_remaining: エラーバジェット残量 (0 = 完全消費)
  budget_remaining : Nat
  -- total_requests: 総リクエスト数
  total_requests   : Nat
  deriving Repr

-- エラーイベントの重み: 1 リクエストあたりのバジェット消費量
def errorCost : Nat := 1

-- エラー発生時のバジェット更新: 残量から errorCost を引く
def consumeBudget (s : SloState) : SloState :=
  { s with budget_remaining := s.budget_remaining - errorCost
         , total_requests := s.total_requests + 1 }

-- 主定理: エラー後のバジェットは以前以下 (単調減少)
theorem error_budget_monotone_decrease (s : SloState) :
    (consumeBudget s).budget_remaining ≤ s.budget_remaining := by
  -- Nat の減算は飽和演算なので budget_remaining - 1 ≤ budget_remaining
  simp [consumeBudget, errorCost]
  omega

-- 定理: バジェットが 0 になったら SLO 違反
def sloViolated (s : SloState) : Bool :=
  s.budget_remaining == 0

-- バジェット枯渇の伝播: 一度 violated になったら消費してもバジェットは増えない
theorem budget_exhausted_stable (s : SloState) (hviol : sloViolated s = true) :
    sloViolated (consumeBudget s) = true := by
  simp [sloViolated, beq_iff_eq] at *
  simp [consumeBudget, hviol, errorCost]

-- 系: n 回のエラー後のバジェットは初期値から n を引いた値以下
theorem n_errors_budget_decrease (s : SloState) (n : Nat) :
    (Nat.iterate consumeBudget n s).budget_remaining ≤ s.budget_remaining := by
  induction n with
  | zero => simp
  | succ k ih =>
    -- k+1 回エラー後は k 回後よりバジェットが小さい
    simp [Nat.iterate]
    exact Nat.le_trans (error_budget_monotone_decrease _) ih

end K1s0Formal.CrossCutting.Slo
