-- Tier1/HLC/Monotone.lean — HLC 単調性 lemma
-- obligation_id: formal_axiom_048 (v1_program_correctness_proof, tool_kind=lean4)
-- statement: HLC advance 操作が常に元の値以上の値を返すことを形式証明する
-- tool: Lean 4.29.1 (Mathlib 不要, omega tactic のみ使用)
-- last_verified_at: 2026-05-23

-- k1s0-proof: PROOF-tier1-lean4-001 -> IMPL-tier1-hlc-001
-- k1s0-impl: IMPL-tier1-hlc-001 realizes=FR-tier1-004

namespace K1s0Formal.Tier1.HLC

-- HLC 型: wall_time（壁時計時刻）と logical（論理カウンタ）のペア
-- client/hlc_lib/rust の Rust 実装に対応する
structure HLC where
  -- wall_time: 物理時刻（単調増加する Nat で表現する）
  wall_time : Nat
  -- logical: 同一 wall_time 内の論理順序カウンタ
  logical : Nat
  deriving Repr

-- HLC の順序関係: (w1, l1) ≤ (w2, l2) ⟺ w1 < w2 ∨ (w1 = w2 ∧ l1 ≤ l2)
-- 辞書式順序で wall_time を優先する
def HLC.le (a b : HLC) : Prop :=
  -- wall_time が小さい方が先に来る
  a.wall_time < b.wall_time ∨
  -- wall_time が等しい場合は logical で比較する
  (a.wall_time = b.wall_time ∧ a.logical ≤ b.logical)

-- HLC.le のインスタンス宣言
instance : LE HLC where
  le := HLC.le

-- HLC advance 操作: now が現在の wall_time より大きければ wall_time を更新、
-- 等しいか小さければ logical をインクリメントする
-- client/hlc_lib/rust の hlc_advance に対応する
def HLC.advance (h : HLC) (now : Nat) : HLC :=
  -- now が現在の wall_time より大きい場合は wall_time を更新して logical をリセットする
  if now > h.wall_time then
    { wall_time := now, logical := 0 }
  -- now が現在の wall_time 以下の場合は logical をインクリメントする
  else
    { wall_time := h.wall_time, logical := h.logical + 1 }

-- 単調性の主定理: HLC.advance の結果は常に元の HLC 以上
-- ∀ h : HLC, ∀ now : Nat, h ≤ h.advance now
-- これが HLC の単調性 (monotonicity) 保証の形式証明
theorem hlc_advance_monotone (h : HLC) (now : Nat) :
    h ≤ h.advance now := by
  -- h.wall_time < now か h.wall_time ≥ now かで場合分けする
  rcases Nat.lt_or_ge h.wall_time now with hlt | hge
  · -- case: h.wall_time < now (now > h.wall_time が成立する場合)
    -- HLC.advance では if 条件が true になり wall_time := now, logical := 0 となる
    simp only [LE.le, HLC.le, HLC.advance, if_pos hlt]
    -- 目標: h.wall_time < now ∨ (h.wall_time = now ∧ h.logical ≤ 0)
    -- h.wall_time < now が既に hlt として成立しているので left を選ぶ
    exact Or.inl hlt
  · -- case: h.wall_time ≥ now (now ≤ h.wall_time, if 条件が false になる場合)
    -- Lean の `>` は `<` の引数を逆にした notation: now > h.wall_time ≡ h.wall_time < now
    -- hge : h.wall_time ≥ now より ¬(h.wall_time < now) が成立する
    have hnlt : ¬(h.wall_time < now) := Nat.not_lt.mpr hge
    -- HLC.advance では if 条件が false になり wall_time := h.wall_time, logical := h.logical + 1 となる
    simp only [LE.le, HLC.le, HLC.advance, if_neg hnlt]
    -- 目標: h.wall_time < h.wall_time ∨ (True ∧ h.logical ≤ h.logical + 1)
    -- simp が h.wall_time = h.wall_time を True に簡約するため trivial を使う
    -- h.logical ≤ h.logical + 1 は Nat.le_add_right で証明できる
    exact Or.inr ⟨trivial, Nat.le_add_right _ _⟩

-- 系: advance 後の wall_time は h.wall_time 以上（主定理から導出する）
-- HLC.le の定義より、hlc_advance_monotone から wall_time の下界が得られる
example (h : HLC) (now : Nat) : h.wall_time ≤ (h.advance now).wall_time := by
  -- hlc_advance_monotone が示す h ≤ h.advance now から wall_time の不等式を取り出す
  have hm := hlc_advance_monotone h now
  -- HLC.le の定義を展開する
  simp only [LE.le, HLC.le] at hm
  -- left: h.wall_time < (h.advance now).wall_time の場合
  -- right: h.wall_time = (h.advance now).wall_time の場合
  rcases hm with h1 | ⟨h2, _⟩
  · -- wall_time が真に増加する場合: h.wall_time < result.wall_time → ≤ が成立する
    exact Nat.le_of_lt h1
  · -- wall_time が変化しない場合: h.wall_time = result.wall_time → ≤ が成立する
    exact h2 ▸ Nat.le_refl _

end K1s0Formal.Tier1.HLC
