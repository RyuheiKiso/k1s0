-- Test/Coverage/Monotonicity.lean — test カバレッジ単調性公理
-- obligation_id: test_axiom_043 (v1_property_axiom, tool_kind=lean4)
-- statement: テストスイートにテストを追加するとカバレッジは単調増加することを証明する
-- tool: Lean 4 (omega / Finset を使用)
-- k1s0-proof: PROOF-test-lean4-001 -> IMPL-test-coverage-001
-- k1s0-impl: IMPL-test-coverage-001 realizes=FR-test-001

namespace K1s0Formal.Test.Coverage

-- コードパス: 自然数で識別する (実装ではソースパスに対応)
abbrev CodePath := Nat

-- テストスイート: カバーされたコードパスの有限集合
-- ここでは List で表現し、後で Finset に変換する
abbrev TestSuite := List CodePath

-- テストが coverage を返す (カバーされたパスの集合)
def coverageOf (suite : TestSuite) : List CodePath := suite.dedup

-- カバレッジサイズ: ユニークなパスの数
def coverageSize (suite : TestSuite) : Nat := (coverageOf suite).length

-- 主定理: テストを追加するとカバレッジは増加するか維持される (≥)
theorem coverage_monotone (suite : TestSuite) (newPaths : TestSuite) :
    coverageSize suite ≤ coverageSize (suite ++ newPaths) := by
  -- coverageOf (suite ++ newPaths) は suite の全パスを含む
  simp [coverageSize, coverageOf]
  -- suite.dedup は (suite ++ newPaths).dedup の部分集合
  -- List.Sublist による証明: suite.dedup は (suite ++ newPaths).dedup の sublist
  apply List.length_le_length_of_sublist
  apply List.dedup_sublist_of_sublist
  exact List.sublist_append_left _ _

-- 補題: 既存パスの重複追加はカバレッジサイズを変えない
theorem duplicate_path_no_change (suite : TestSuite) (path : CodePath)
    (hmem : path ∈ suite) :
    coverageSize suite = coverageSize (suite ++ [path]) := by
  -- path が既に suite に含まれるので dedup 後のサイズは変わらない
  simp [coverageSize, coverageOf]
  apply List.length_dedup_eq_of_mem_append
  exact hmem

-- 系: 新しいパスを追加するとカバレッジサイズが厳密に増加する
theorem new_path_increases_coverage (suite : TestSuite) (path : CodePath)
    (hnew : path ∉ coverageOf suite) :
    coverageSize suite < coverageSize (suite ++ [path]) := by
  -- path が coverageOf suite にないので dedup 後のリストに path が追加される
  simp [coverageSize, coverageOf] at *
  -- (suite ++ [path]).dedup は suite.dedup ++ [path] になる
  -- (path が suite.dedup にない場合)
  have hsub : suite.dedup.length < (suite ++ [path]).dedup.length := by
    apply List.length_lt_of_not_mem_dedup_append
    exact hnew
  exact hsub

end K1s0Formal.Test.Coverage
