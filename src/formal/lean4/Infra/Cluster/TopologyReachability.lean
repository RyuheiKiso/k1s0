-- Infra/Cluster/TopologyReachability.lean — infra クラスタトポロジー到達可能性公理
-- obligation_id: infra_axiom_019 (v1_property_axiom, tool_kind=lean4)
-- statement: 到達可能性関係は推移閉包を取っても増加単調であることを証明する
-- tool: Lean 4 (List membership axioms を使用)
-- k1s0-proof: PROOF-infra-lean4-001 -> IMPL-infra-topology-001
-- k1s0-impl: IMPL-infra-topology-001 realizes=FR-infra-001

namespace K1s0Formal.Infra.Cluster

-- ノード型 (自然数で識別する)
abbrev NodeId := Nat

-- エッジ型: (from, to) のペア
abbrev Edge := NodeId × NodeId

-- グラフ: エッジのリスト
abbrev Graph := List Edge

-- 直接到達可能: g に (a, b) エッジが存在する
def directlyReachable (g : Graph) (a b : NodeId) : Prop :=
  (a, b) ∈ g

-- 推移的到達可能: ステップ数 n の帰納的定義
-- n=0: a = b (自己到達)
-- n+1: ∃ c, directlyReachable a c ∧ reachableIn n c b
def reachableIn : Nat → Graph → NodeId → NodeId → Prop
  | 0,     _,  a, b => a = b
  | n + 1, g,  a, b => ∃ c, directlyReachable g a c ∧ reachableIn n g c b

-- 到達可能性: ある n でステップ数 n 以内に到達できる
def reachable (g : Graph) (a b : NodeId) : Prop :=
  ∃ n, reachableIn n g a b

-- 主定理: 到達可能性は反射的
theorem reachable_refl (g : Graph) (a : NodeId) : reachable g a a :=
  ⟨0, rfl⟩

-- 主定理: 到達可能性は推移的
theorem reachable_trans (g : Graph) (a b c : NodeId)
    (hab : reachable g a b) (hbc : reachable g b c) : reachable g a c := by
  -- a から b までの到達ステップ数 n1 を取得する
  obtain ⟨n1, h1⟩ := hab
  -- b から c までの到達ステップ数 n2 を取得する
  obtain ⟨n2, h2⟩ := hbc
  -- n1 + n2 ステップで a から c に到達できることを証明する
  refine ⟨n1 + n2, ?_⟩
  -- reachableIn を n1 についての帰納法で証明する
  induction n1 generalizing a with
  | zero =>
    -- n1 = 0 の場合: a = b なので b から c に到達できれば a から c に到達できる
    simp [reachableIn] at h1 ⊢
    subst h1
    exact h2
  | succ k ih =>
    -- n1 = k + 1 の場合: 中間ノード d を介して到達する
    obtain ⟨d, hd, hdk⟩ := h1
    -- d から c に到達できることを帰納法で証明する
    exact ⟨d, hd, ih hdk⟩

-- 系: グラフにエッジを追加すると到達可能性は増加単調
theorem reachable_monotone (g : Graph) (e : Edge) (a b : NodeId)
    (hr : reachable g a b) : reachable (e :: g) a b := by
  -- g での到達ステップ数 n を取得する
  obtain ⟨n, hn⟩ := hr
  -- e :: g でも同じステップ数で到達できることを示す
  refine ⟨n, ?_⟩
  -- n についての帰納法で証明する
  induction n generalizing a with
  | zero =>
    -- a = b なので trivial
    exact hn
  | succ k ih =>
    -- 中間ノード c を介して到達する
    obtain ⟨c, hc, hck⟩ := hn
    -- e :: g でも hc のエッジが存在する
    exact ⟨c, List.mem_cons_of_mem _ hc, ih hck⟩

end K1s0Formal.Infra.Cluster
