-- Security/AccessControl/DenyPrecedence.lean — security アクセス制御 deny 優先公理
-- obligation_id: security_axiom_028 (v1_property_axiom, tool_kind=lean4)
-- statement: 明示的 deny は全ての allow より優先されることを証明する
-- tool: Lean 4 (simp / decide を使用)
-- k1s0-proof: PROOF-security-lean4-001 -> IMPL-security-access-001
-- k1s0-impl: IMPL-security-access-001 realizes=FR-security-001

namespace K1s0Formal.Security.AccessControl

-- アクセス制御決定
inductive Decision where
  -- アクセス許可
  | Allow : Decision
  -- アクセス拒否 (理由コード付き)
  | Deny  : Nat → Decision
  deriving Repr, DecidableEq

-- ポリシーリスト: 各ポリシーは決定を返す関数 (抽象化)
-- 実装では OPA Rego や Kyverno policy に対応する
structure PolicyResult where
  -- policy_id: ポリシーの識別子
  policy_id : Nat
  -- decision: このポリシーの決定
  decision  : Decision
  deriving Repr

-- deny-override: deny が 1 件でもあれば全体を deny にする
def combineDecisions (results : List PolicyResult) : Decision :=
  -- deny が存在する場合はその deny を返す (最初の deny を採用)
  match results.find? (fun r => match r.decision with | Decision.Deny _ => true | _ => false) with
  | some r => r.decision
  | none   => Decision.Allow

-- 主定理: deny が存在すれば combineDecisions は Deny を返す
theorem deny_takes_precedence (results : List PolicyResult) (deny_result : PolicyResult)
    (reason : Nat) (hd : deny_result.decision = Decision.Deny reason)
    (hmem : deny_result ∈ results) :
    ∃ r, combineDecisions results = Decision.Deny r := by
  -- combineDecisions の定義を展開する
  simp [combineDecisions]
  -- deny_result が results に含まれるので find? は some を返す
  have hfind : results.find? (fun r => match r.decision with
      | Decision.Deny _ => true | _ => false) = some deny_result := by
    apply List.find?_some_of_mem hmem
    simp [hd]
  rw [hfind]
  -- deny_result.decision は Deny reason なのでそれを返す
  exact ⟨reason, hd⟩

-- 系: allow のみの場合は Allow を返す
theorem all_allow_combines_to_allow (results : List PolicyResult)
    (hall : ∀ r ∈ results, r.decision = Decision.Allow) :
    combineDecisions results = Decision.Allow := by
  -- combineDecisions の定義を展開する
  simp [combineDecisions]
  -- find? の条件に一致する要素がないことを示す
  have hno : results.find? (fun r => match r.decision with
      | Decision.Deny _ => true | _ => false) = none := by
    apply List.find?_eq_none.mpr
    intro r hmem
    -- r.decision = Allow なので条件は false
    have hd := hall r hmem
    simp [hd]
  rw [hno]

-- 補題: deny-first は交換可能 (順序に依存しない)
-- deny が存在する場合は結果は Deny になる (順序不問)
theorem deny_order_invariant (r1 r2 : PolicyResult) (reason : Nat)
    (hd : r1.decision = Decision.Deny reason) :
    ∃ r, combineDecisions [r1, r2] = Decision.Deny r ∧
         combineDecisions [r2, r1] = Decision.Deny r := by
  -- r1 が deny なので両方向で Deny が返ることを示す
  have h1 : ∃ r, combineDecisions [r1, r2] = Decision.Deny r := by
    exact deny_takes_precedence [r1, r2] r1 reason hd (List.mem_cons_self _ _)
  obtain ⟨r, hr⟩ := h1
  have h2 : ∃ r', combineDecisions [r2, r1] = Decision.Deny r' := by
    exact deny_takes_precedence [r2, r1] r1 reason hd (List.mem_cons_of_mem _ (List.mem_cons_self _ _))
  obtain ⟨r', hr'⟩ := h2
  -- 両方の結果が同じ reason の Deny であることを確認する
  simp [combineDecisions, hd] at hr hr'
  exact ⟨r, hr, hr'⟩

end K1s0Formal.Security.AccessControl
