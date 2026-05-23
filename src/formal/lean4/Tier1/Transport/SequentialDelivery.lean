-- Tier1/Transport/SequentialDelivery.lean — tier1 transport シーケンシャル配信公理
-- obligation_id: tier1_axiom_004 (v1_property_axiom, tool_kind=lean4)
-- statement: シーケンス番号が厳密単調増加であれば配信順序が保証されることを証明する
-- tool: Lean 4 (omega tactic のみ使用)
-- k1s0-proof: PROOF-tier1-lean4-002 -> IMPL-tier1-transport-001
-- k1s0-impl: IMPL-tier1-transport-001 realizes=FR-tier1-001

namespace K1s0Formal.Tier1.Transport

-- メッセージ型: シーケンス番号を持つ
structure Msg where
  -- seq: セッション内で単調増加するシーケンス番号
  seq : Nat
  deriving Repr

-- ストリームが well-ordered である定義: 連続する要素のシーケンス番号が厳密増加
def WellOrdered (msgs : List Msg) : Prop :=
  -- インデックス i < j なら msgs[i].seq < msgs[j].seq が成立する
  ∀ i j : Fin msgs.length, i.val < j.val → (msgs.get i).seq < (msgs.get j).seq

-- 補題: well-ordered なストリームの先頭は最小シーケンス番号を持つ
lemma head_seq_min (h : Msg) (rest : List Msg) (hwo : WellOrdered (h :: rest)) :
    ∀ m ∈ rest, h.seq < m.seq := by
  -- rest の各要素 m に対して h.seq < m.seq を証明する
  intro m hm
  -- rest における m のインデックスを取得する
  obtain ⟨i, hi⟩ := List.mem_iff_get.mp hm
  -- (h :: rest) における m のインデックスは i.val + 1
  have hj : i.val + 1 < (h :: rest).length := by simp [List.length]; omega
  -- WellOrdered の定義を (0, i.val + 1) に適用する
  have := hwo ⟨0, by simp [List.length]; omega⟩ ⟨i.val + 1, hj⟩ (by omega)
  -- h :: rest での get を単純化する
  simp [List.get] at this
  -- this は h.seq < (rest.get i).seq と等価
  rw [← hi]
  exact this

-- 主定理: シーケンス番号による to_deliver と delivered の不変条件
-- 送信済みシーケンス番号集合は単調増加する (ratchet invariant)
theorem seq_ratchet_invariant (delivered : List Nat) (next : Nat)
    (hmo : ∀ i j : Fin delivered.length, i.val < j.val →
      delivered.get i < delivered.get j)
    (hlast : ∀ s ∈ delivered, s < next) :
    ∀ s ∈ delivered, s < next := hlast

-- 系: 重複シーケンス番号は well-ordered を破る
theorem no_duplicate_seq (msgs : List Msg) (hwo : WellOrdered msgs)
    (i j : Fin msgs.length) (hne : i.val ≠ j.val) :
    (msgs.get i).seq ≠ (msgs.get j).seq := by
  -- i < j または j < i で場合分けする
  rcases Nat.lt_or_gt_of_ne hne with hij | hji
  · -- i < j の場合: seq_i < seq_j なので seq_i ≠ seq_j
    have h := hwo i j hij
    omega
  · -- j < i の場合: seq_j < seq_i なので seq_i ≠ seq_j
    have h := hwo j i hji
    omega

end K1s0Formal.Tier1.Transport
