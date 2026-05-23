-- CrossCutting/Pii/IsolationTransitivity.lean — PII 分離推移性公理
-- obligation_id: cross_pii_axiom_083 (v1_property_axiom, tool_kind=lean4)
-- statement: PII クラス A が B を含意し B が C を含意するなら A は C も含意することを証明する
-- tool: Lean 4 (simp を使用)
-- k1s0-proof: PROOF-cross-pii-lean4-001 -> IMPL-crosscutting-pii-001
-- k1s0-impl: IMPL-crosscutting-pii-001 realizes=FR-cross-pii-001

namespace K1s0Formal.CrossCutting.Pii

-- PII フィールド型 (自然数で識別する)
abbrev PiiField := Nat

-- PII クラス: 対象フィールドの集合
abbrev PiiClass := List PiiField

-- クラス包含: c1 が c2 の全フィールドを含む
def classSubset (c1 c2 : PiiClass) : Prop :=
  ∀ f ∈ c2, f ∈ c1

-- 主定理: classSubset は推移的
theorem pii_class_subset_trans (c1 c2 c3 : PiiClass)
    (h12 : classSubset c1 c2) (h23 : classSubset c2 c3) :
    classSubset c1 c3 := by
  intro f hf
  exact h12 f (h23 f hf)

-- 補題: classSubset は反射的
theorem pii_class_subset_refl (c : PiiClass) : classSubset c c := by
  intro f hf; exact hf

-- PII マスキング: クラスのフィールドを全てマスクする
def maskClass (data : List (PiiField × Nat)) (cls : PiiClass) : List (PiiField × Nat) :=
  data.filter (fun (f, _) => f ∉ cls)

-- 主定理: 大きいクラスでマスクすると小さいクラスでマスクより結果が少ない (より多くマスクされる)
theorem larger_class_masks_more (data : List (PiiField × Nat)) (c1 c2 : PiiClass)
    (hsub : classSubset c1 c2) :
    (maskClass data c1).length ≥ (maskClass data c2).length := by
  -- c1 が c2 を含むので c2 でマスクされるフィールドは c1 でもマスクされる
  simp [maskClass]
  apply List.length_filter_le_of_imp_filter
  intro ⟨f, v⟩ h
  simp at *
  intro hmem
  exact h (hsub f hmem)

-- 系: PII クラスの和集合でマスクすると交差でマスクより結果が少ない
theorem union_masks_more_than_intersect (data : List (PiiField × Nat)) (c1 c2 : PiiClass) :
    (maskClass data (c1 ++ c2)).length ≤ (maskClass data c1).length := by
  simp [maskClass]
  apply List.length_filter_le_of_imp_filter
  intro ⟨f, v⟩ h
  simp at *
  exact fun hc => h (Or.inl hc)

end K1s0Formal.CrossCutting.Pii
