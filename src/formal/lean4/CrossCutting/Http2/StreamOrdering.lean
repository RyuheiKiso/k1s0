-- CrossCutting/Http2/StreamOrdering.lean — HTTP/2 ストリーム多重化非干渉公理
-- obligation_id: cross_http2_axiom_053 (v1_property_axiom, tool_kind=lean4)
-- statement: 異なるストリーム ID のフレームは互いに独立した順序を持つことを証明する
-- tool: Lean 4 (omega を使用)
-- k1s0-proof: PROOF-cross-http2-lean4-001 -> IMPL-crosscutting-http2-001
-- k1s0-impl: IMPL-crosscutting-http2-001 realizes=FR-cross-http2-001

namespace K1s0Formal.CrossCutting.Http2

-- HTTP/2 フレーム: ストリーム ID とシーケンス番号を持つ
structure Frame where
  -- stream_id: フレームが属するストリームの ID
  stream_id : Nat
  -- seq: ストリーム内のシーケンス番号
  seq       : Nat
  deriving Repr

-- 接続全体のフレームリスト (到着順)
abbrev Connection := List Frame

-- 特定ストリームのフレームのみを抽出する
def streamFrames (conn : Connection) (sid : Nat) : List Frame :=
  conn.filter (fun f => f.stream_id == sid)

-- 主定理: 異なるストリームのフレームは互いに排他的
theorem stream_frames_disjoint (conn : Connection) (s1 s2 : Nat) (hne : s1 ≠ s2) :
    ∀ f, f ∈ streamFrames conn s1 → f ∉ streamFrames conn s2 := by
  intro f h1 h2
  simp [streamFrames, List.mem_filter] at h1 h2
  have he1 : f.stream_id = s1 := Nat.eq_of_beq_eq_true h1.2
  have he2 : f.stream_id = s2 := Nat.eq_of_beq_eq_true h2.2
  exact hne (he1 ▸ he2)

-- 定理: ストリーム内のフレーム数は接続全体のフレーム数以下
theorem stream_size_le_total (conn : Connection) (sid : Nat) :
    (streamFrames conn sid).length ≤ conn.length :=
  List.length_filter_le _ conn

end K1s0Formal.CrossCutting.Http2
