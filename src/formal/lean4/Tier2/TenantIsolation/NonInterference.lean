-- Tier2/TenantIsolation/NonInterference.lean — tier2 テナント分離非干渉公理
-- obligation_id: tier2_axiom_009 (v1_property_axiom, tool_kind=lean4)
-- statement: 異なるテナント ID を持つ操作は互いに干渉しないことを証明する
-- tool: Lean 4 (decide / omega tactic を使用)
-- k1s0-proof: PROOF-tier2-lean4-001 -> IMPL-tier2-tenant-001
-- k1s0-impl: IMPL-tier2-tenant-001 realizes=FR-tier2-001

namespace K1s0Formal.Tier2.TenantIsolation

-- テナント ID 型 (自然数で表現する)
abbrev TenantId := Nat

-- データ行: テナント ID とペイロードを持つ
structure Row where
  -- tenant_id: この行を所有するテナントの ID
  tenant_id : TenantId
  -- payload: 行データ (自然数で抽象化する)
  payload   : Nat
  deriving Repr, DecidableEq

-- フィルタ: 指定テナントの行のみを返す (PostgreSQL RLS に対応)
def filterByTenant (rows : List Row) (tid : TenantId) : List Row :=
  rows.filter (fun r => r.tenant_id == tid)

-- 主定理: 異なるテナント ID でフィルタすると結果は互いに素
theorem tenant_filter_disjoint (rows : List Row) (t1 t2 : TenantId) (hne : t1 ≠ t2) :
    ∀ r, r ∈ filterByTenant rows t1 → r ∉ filterByTenant rows t2 := by
  -- 任意の行 r について
  intro r hr1 hr2
  -- hr1: r ∈ filterByTenant rows t1 より r.tenant_id = t1
  simp [filterByTenant, List.mem_filter] at hr1 hr2
  -- hr1.right: r.tenant_id == t1 (BEq)
  -- hr2.right: r.tenant_id == t2 (BEq)
  have h1 : r.tenant_id = t1 := by exact Nat.eq_of_beq_eq_true hr1.2
  have h2 : r.tenant_id = t2 := by exact Nat.eq_of_beq_eq_true hr2.2
  -- r.tenant_id = t1 かつ r.tenant_id = t2 なら t1 = t2 が導かれ hne と矛盾する
  exact hne (h1 ▸ h2)

-- 補題: フィルタ結果のテナント ID は全て指定値に一致する
lemma filter_tenant_id_eq (rows : List Row) (tid : TenantId) :
    ∀ r ∈ filterByTenant rows tid, r.tenant_id = tid := by
  -- 任意の行 r について
  intro r hr
  -- filterByTenant の定義を展開する
  simp [filterByTenant, List.mem_filter] at hr
  -- hr.right: r.tenant_id == tid (BEq) から等式を取り出す
  exact Nat.eq_of_beq_eq_true hr.2

-- 系: テナント A の書き込みはテナント B の読み込み結果に影響しない
theorem write_non_interference (rows : List Row) (newRow : Row) (reader : TenantId)
    (hne : newRow.tenant_id ≠ reader) :
    filterByTenant (newRow :: rows) reader = filterByTenant rows reader := by
  -- filterByTenant の定義を展開して newRow.tenant_id ≠ reader を使う
  simp [filterByTenant, List.filter]
  -- newRow.tenant_id == reader は false なのでリストに追加されない
  have : ¬(newRow.tenant_id == reader) := by
    simp [beq_iff_eq]
    exact hne
  simp [this]

end K1s0Formal.Tier2.TenantIsolation
