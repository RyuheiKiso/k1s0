-- Tier3/ClientState/PendingQueueOrder.lean — tier3 クライアント状態 pending queue 順序公理
-- obligation_id: tier3_axiom_014 (v1_property_axiom, tool_kind=lean4)
-- statement: pending queue の先入れ先出し (FIFO) 順序が保証されることを証明する
-- tool: Lean 4 (List append axioms を使用)
-- k1s0-proof: PROOF-tier3-lean4-001 -> IMPL-tier3-state-001
-- k1s0-impl: IMPL-tier3-state-001 realizes=FR-tier3-001

namespace K1s0Formal.Tier3.ClientState

-- pending queue エントリ: idempotency key とペイロードを持つ
structure PQEntry where
  -- idempotency_key: 重複排除に使用するキー
  idempotency_key : Nat
  -- payload: 操作データ (自然数で抽象化する)
  payload         : Nat
  deriving Repr, DecidableEq

-- pending queue: エントリのリスト (先頭が最も古いエントリ)
abbrev PendingQueue := List PQEntry

-- enqueue: 末尾にエントリを追加する
def enqueue (pq : PendingQueue) (e : PQEntry) : PendingQueue := pq ++ [e]

-- dequeue_head: 先頭エントリを取得して残りのキューを返す
def dequeue_head (pq : PendingQueue) : Option (PQEntry × PendingQueue) :=
  match pq with
  | []      => none
  | h :: t  => some (h, t)

-- 主定理: enqueue してから dequeue すると、元の先頭要素が dequeue される (FIFO 不変条件)
-- 空でないキューで enqueue → dequeue は元の先頭要素を返す
theorem fifo_order (pq : PendingQueue) (e : PQEntry) (h : PQEntry) (t : PendingQueue)
    (hpq : pq = h :: t) :
    dequeue_head (enqueue pq e) = some (h, t ++ [e]) := by
  -- enqueue と dequeue_head の定義を展開する
  simp [enqueue, dequeue_head, hpq]
  -- hpq を代入すると (h :: t) ++ [e] = h :: (t ++ [e])
  -- dequeue_head (h :: (t ++ [e])) = some (h, t ++ [e]) が得られる
  rfl

-- 補題: 空でないキューの enqueue は空でない
lemma enqueue_nonempty (pq : PendingQueue) (e : PQEntry) : enqueue pq e ≠ [] := by
  simp [enqueue]

-- 補題: idempotency_key による重複検出
-- 同じキーのエントリが既に存在する場合は true を返す
def hasDuplicate (pq : PendingQueue) (key : Nat) : Bool :=
  pq.any (fun e => e.idempotency_key == key)

-- 定理: 重複のないキューにエントリを追加すると、重複チェックが成立する
theorem duplicate_detection (pq : PendingQueue) (e : PQEntry)
    (hno : ¬hasDuplicate pq e.idempotency_key) :
    hasDuplicate (enqueue pq e) e.idempotency_key = true := by
  -- enqueue の定義を展開する
  simp [enqueue, hasDuplicate, List.any_append, beq_iff_eq]

end K1s0Formal.Tier3.ClientState
