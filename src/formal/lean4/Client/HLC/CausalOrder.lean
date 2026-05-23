-- Client/HLC/CausalOrder.lean — client HLC 因果順序公理
-- obligation_id: client_axiom_038 (v1_property_axiom, tool_kind=lean4)
-- statement: HLC の因果順序が全順序の部分順序を形成することを証明する
-- tool: Lean 4 (omega を使用)
-- k1s0-proof: PROOF-client-lean4-001 -> IMPL-client-hlc-001
-- k1s0-impl: IMPL-client-hlc-001 realizes=FR-client-001

namespace K1s0Formal.Client.HLC

-- HLC 型 (Tier1/HLC/Monotone.lean と同一定義)
structure HLC where
  -- wall_time: 物理時刻
  wall_time : Nat
  -- logical: 論理カウンタ
  logical   : Nat
  deriving Repr

-- HLC の全順序比較: 辞書式順序
def HLC.lt (a b : HLC) : Prop :=
  -- wall_time が小さい方が先か、同じなら logical が小さい方が先
  a.wall_time < b.wall_time ∨
  (a.wall_time = b.wall_time ∧ a.logical < b.logical)

-- HLC の等価性
def HLC.equiv (a b : HLC) : Prop :=
  a.wall_time = b.wall_time ∧ a.logical = b.logical

-- 主定理: HLC.lt は推移的
theorem hlc_lt_trans (a b c : HLC) (hab : HLC.lt a b) (hbc : HLC.lt b c) :
    HLC.lt a c := by
  -- HLC.lt の定義を展開する
  simp [HLC.lt] at *
  -- 4 つの場合分け
  rcases hab with h1 | ⟨h2, h3⟩ <;> rcases hbc with h4 | ⟨h5, h6⟩
  · -- a.wall < b.wall ∧ b.wall < c.wall → a.wall < c.wall
    exact Or.inl (Nat.lt_trans h1 h4)
  · -- a.wall < b.wall ∧ b.wall = c.wall → a.wall < c.wall
    exact Or.inl (h5 ▸ h1)
  · -- a.wall = b.wall ∧ b.wall < c.wall → a.wall < c.wall
    exact Or.inl (h2 ▸ h4)
  · -- a.wall = b.wall ∧ b.wall = c.wall ∧ a.logical < b.logical ∧ b.logical < c.logical
    exact Or.inr ⟨h2.trans h5, Nat.lt_trans h3 h6⟩

-- 主定理: HLC.lt は非反射的 (irreflexive)
theorem hlc_lt_irrefl (a : HLC) : ¬HLC.lt a a := by
  simp [HLC.lt]
  omega

-- 主定理: HLC.lt は三分法を満たす (totality)
theorem hlc_lt_total_or_equiv (a b : HLC) :
    HLC.lt a b ∨ HLC.equiv a b ∨ HLC.lt b a := by
  simp [HLC.lt, HLC.equiv]
  -- 自然数の全順序から場合分けする
  rcases Nat.lt_trichotomy a.wall_time b.wall_time with h | h | h
  · exact Or.inl (Or.inl h)
  · rcases Nat.lt_trichotomy a.logical b.logical with hl | hl | hl
    · exact Or.inl (Or.inr ⟨h, hl⟩)
    · exact Or.inr (Or.inl ⟨h, hl⟩)
    · exact Or.inr (Or.inr (Or.inr ⟨h.symm, hl⟩))
  · exact Or.inr (Or.inr (Or.inl h))

-- 因果関係: a は b よりも先に発生した (happened-before)
-- Lamport の happened-before の HLC 版
def happenedBefore (a b : HLC) : Prop := HLC.lt a b

-- 定理: happened-before は因果順序の公理を満たす
-- (1) 不反射性
theorem hb_irrefl (a : HLC) : ¬happenedBefore a a :=
  hlc_lt_irrefl a

-- (2) 推移性
theorem hb_trans (a b c : HLC) (h1 : happenedBefore a b) (h2 : happenedBefore b c) :
    happenedBefore a c :=
  hlc_lt_trans a b c h1 h2

-- (3) 非対称性: a → b ならば b ↛ a
theorem hb_asymm (a b : HLC) (h : happenedBefore a b) : ¬happenedBefore b a := by
  intro h'
  -- h と h' が矛盾することを示す
  have := hlc_lt_trans a b a h h'
  exact hlc_lt_irrefl a this

end K1s0Formal.Client.HLC
