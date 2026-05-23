-- CrossCutting/Edge/SyncIdempotency.lean — ops edge クラスタ同期冪等性公理
-- obligation_id: cross_edge_axiom_088 (v1_property_axiom, tool_kind=lean4)
-- statement: 同じ設定を 2 回同期しても結果は 1 回同期した場合と同じことを証明する
-- tool: Lean 4 (simp を使用)
-- k1s0-proof: PROOF-cross-edge-lean4-001 -> IMPL-crosscutting-edge-001
-- k1s0-impl: IMPL-crosscutting-edge-001 realizes=FR-cross-edge-001

namespace K1s0Formal.CrossCutting.Edge

-- クラスタ設定型 (key-value マップを List で表現)
abbrev ConfigKey   := Nat
abbrev ConfigValue := Nat
abbrev ClusterConfig := List (ConfigKey × ConfigValue)

-- 設定のマージ: 後の設定が優先 (GitOps の apply 操作に対応)
def applyConfig (current : ClusterConfig) (desired : ClusterConfig) : ClusterConfig :=
  -- desired の key-value が current を上書きする
  -- 簡略化: desired を current にマージして desired の key を優先する
  let desired_keys := desired.map Prod.fst
  -- current から desired_keys を除去した後、desired を追加する
  let filtered := current.filter (fun (k, _) => k ∉ desired_keys)
  filtered ++ desired

-- 主定理: 同じ設定を 2 回 apply しても結果は変わらない (冪等性)
theorem apply_config_idempotent (current : ClusterConfig) (desired : ClusterConfig)
    (hno_dup : desired.map Prod.fst = (desired.map Prod.fst).dedup) :
    applyConfig (applyConfig current desired) desired = applyConfig current desired := by
  -- applyConfig の定義を展開する
  simp [applyConfig]
  -- 2 回目の apply で desired_keys のフィルタは desired 自身に対して適用される
  -- desired の key は desired_keys に全て含まれるので filtered は空になる
  have hfilter : (desired.filter (fun pair => pair.fst ∉ desired.map Prod.fst)).length = 0 := by
    simp [List.filter_eq_nil_iff]
    intro a ha
    simp
    exact List.mem_map_of_mem Prod.fst ha
  rw [List.length_eq_zero] at hfilter
  rw [hfilter]
  simp

-- 補題: apply は単調 (適用後は desired の全設定が含まれる)
theorem apply_contains_desired (current : ClusterConfig) (desired : ClusterConfig)
    (key : ConfigKey) (val : ConfigValue) (hmem : (key, val) ∈ desired) :
    (key, val) ∈ applyConfig current desired := by
  simp [applyConfig]
  exact List.mem_append_right _ hmem

end K1s0Formal.CrossCutting.Edge
