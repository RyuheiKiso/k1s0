-- CrossCutting/Schema/BackwardCompat.lean — スキーマ進化後方互換性公理
-- obligation_id: cross_schema_axiom_063 (v1_property_axiom, tool_kind=lean4)
-- statement: スキーマにオプションフィールドのみ追加する場合は後方互換性が保たれることを証明する
-- tool: Lean 4 (simp を使用)
-- k1s0-proof: PROOF-cross-schema-lean4-001 -> IMPL-crosscutting-schema-001
-- k1s0-impl: IMPL-crosscutting-schema-001 realizes=FR-cross-schema-001

namespace K1s0Formal.CrossCutting.Schema

-- フィールド定義
structure FieldDef where
  -- name: フィールド名
  name     : String
  -- required: 必須フィールドかどうか
  required : Bool
  deriving Repr, DecidableEq

-- スキーマ: フィールドの有限集合
abbrev Schema := List FieldDef

-- 後方互換性の定義:
-- v2 は v1 と後方互換 ⟺ v1 の必須フィールドが全て v2 にも存在する
def backwardCompat (v1 v2 : Schema) : Prop :=
  ∀ f ∈ v1, f.required = true → f ∈ v2

-- 主定理: オプションフィールドのみ追加した場合は後方互換性が保たれる
theorem optional_addition_compat (v1 : Schema) (newField : FieldDef)
    (hopt : newField.required = false) :
    backwardCompat v1 (v1 ++ [newField]) := by
  -- 任意の必須フィールド f について v1 ++ [newField] にも含まれることを示す
  intro f hf _
  -- f は v1 に含まれるので v1 ++ [newField] にも含まれる
  exact List.mem_append_left [newField] hf

-- 補題: 必須フィールドの削除は後方互換性を破る
theorem required_removal_breaks_compat (v1 : Schema) (f : FieldDef)
    (hreq : f.required = true) (hmem : f ∈ v1) :
    ¬backwardCompat v1 (v1.erase f) := by
  -- f が v1.erase f にない場合に後方互換性が破れることを示す
  intro hcompat
  -- backwardCompat の定義から f ∈ v1.erase f が導かれるはず
  have h := hcompat f hmem hreq
  -- しかし f は erase されているので v1.erase f には含まれない
  have hnotin : f ∉ v1.erase f := List.not_mem_erase f v1
  exact hnotin h

-- 系: 後方互換性は推移的
theorem compat_trans (v1 v2 v3 : Schema)
    (h12 : backwardCompat v1 v2) (h23 : backwardCompat v2 v3) :
    backwardCompat v1 v3 := by
  intro f hf hreq
  exact h23 f (h12 f hf hreq) hreq

end K1s0Formal.CrossCutting.Schema
