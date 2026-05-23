-- Meta/AxisRegistry/CapInvariant.lean — meta 軸レジストリ上限不変条件公理
-- obligation_id: meta_axiom_093 (v1_property_axiom, tool_kind=lean4)
-- statement: 軸レジストリの登録数が cap 以下を維持する不変条件を証明する
-- tool: Lean 4 (omega を使用)
-- k1s0-proof: PROOF-meta-lean4-001 -> IMPL-meta-registry-001
-- k1s0-impl: IMPL-meta-registry-001 realizes=FR-meta-001

namespace K1s0Formal.Meta.AxisRegistry

-- 軸定義
structure Axis where
  -- axis_id: 軸の識別子 (canonical axis_id)
  axis_id   : String
  -- layer: 軸が属するレイヤー (主要/cross-cutting/meta)
  layer     : String
  deriving Repr

-- 軸レジストリ
structure AxisRegistry where
  -- axes: 登録済みの軸リスト
  axes      : List Axis
  -- cap: 登録可能な最大軸数
  cap       : Nat
  -- inv: 現在の登録数が cap 以下という不変条件
  inv       : axes.length ≤ cap
  deriving Repr

-- 軸を追加できるかどうかを確認する
def canAddAxis (r : AxisRegistry) : Bool :=
  r.axes.length < r.cap

-- 主定理: cap 以下の場合のみ軸の追加が成功する
theorem add_axis_respects_cap (r : AxisRegistry) (a : Axis)
    (hcan : canAddAxis r = true) :
    (r.axes ++ [a]).length ≤ r.cap := by
  -- canAddAxis が true なら r.axes.length < r.cap
  simp [canAddAxis] at hcan
  -- r.axes ++ [a] の長さは r.axes.length + 1
  simp [List.length_append]
  omega

-- 補題: cap を超えた追加は不変条件を破る
theorem exceeds_cap_violates_inv (r : AxisRegistry)
    (hfull : canAddAxis r = false) (a : Axis) :
    ¬((r.axes ++ [a]).length ≤ r.cap) := by
  simp [canAddAxis] at hfull
  simp [List.length_append]
  omega

-- k1s0 v1 の具体的な不変条件: cap = 20, 現在 19 軸, 残 1
def k1s0RegistryCap : Nat := 20
def k1s0CurrentAxes : Nat := 19

-- k1s0 現在の残余スロット
theorem k1s0_has_one_slot_remaining :
    k1s0CurrentAxes < k1s0RegistryCap := by
  -- 19 < 20 を omega で証明する
  simp [k1s0CurrentAxes, k1s0RegistryCap]
  omega

-- k1s0 cap を超えると v1 範囲での変更は破壊的
theorem k1s0_exceeds_cap_after_two_additions :
    k1s0CurrentAxes + 2 > k1s0RegistryCap := by
  simp [k1s0CurrentAxes, k1s0RegistryCap]
  omega

-- 系: 重複した axis_id は登録できない (レジストリの一意性)
def hasAxisId (axes : List Axis) (id : String) : Bool :=
  axes.any (fun a => a.axis_id == id)

theorem unique_axis_id_invariant (axes : List Axis) (a : Axis)
    (hno_dup : ¬hasAxisId axes a.axis_id) :
    ∀ b ∈ axes, b.axis_id ≠ a.axis_id := by
  intro b hmem heq
  apply hno_dup
  simp [hasAxisId, List.any_eq_true]
  exact ⟨b, hmem, by simp [beq_iff_eq, heq]⟩

end K1s0Formal.Meta.AxisRegistry
