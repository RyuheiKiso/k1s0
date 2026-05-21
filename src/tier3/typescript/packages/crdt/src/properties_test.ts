// k1s0 tier3 CRDT 収束法則プロパティ検証テスト
// properties.ts の isCommutative / isAssociative / isIdempotent を
// 具体例で検証し、CRDT の 3 法則（交換律・結合律・冪等律）が実装されていることを確認する
// 11_クライアント状態適合仕様.md §CRDT merge invariant: commutative / associative / idempotent
// 参考: "A comprehensive study of CRDTs" (Shapiro et al., 2011)

// 検証対象の property 関数を import する
import {
  // isCommutative: 交換律の検証関数
  isCommutative,
  // isAssociative: 結合律の検証関数
  isAssociative,
  // isIdempotent: 冪等律の検証関数
  isIdempotent,
  // checkCRDTConvergence: 3 法則一括検証関数
  checkCRDTConvergence,
} from "./properties.js";

// ---- テスト対象 CRDT: 最大値 LWW（Last-Write-Wins）レジスタ ----
// value が大きい方が勝つ LWW レジスタ。交換律・結合律・冪等律を全て満たす。

// LWWRecord: LWW レジスタの状態型
// value: 保持する値
// timestamp: HLC タイムスタンプ（wall-clock ではなく HLC ベース）
type LWWRecord = {
  // 保持する数値
  readonly value: number;
  // HLC タイムスタンプ（大きい方が新しい）
  readonly timestamp: number;
};

// lwwMerge: 2 つの LWW レジスタをマージする関数（timestamp が大きい方を優先する）
// タイムスタンプが等しい場合は value が大きい方を優先する（冪等性を保証する）
function lwwMerge(a: LWWRecord, b: LWWRecord): LWWRecord {
  // b のタイムスタンプが a より大きい場合は b を返す
  if (b.timestamp > a.timestamp) {
    // b が新しいので b を返す
    return b;
  }
  // a のタイムスタンプが b より大きい場合は a を返す
  if (a.timestamp > b.timestamp) {
    // a が新しいので a を返す
    return a;
  }
  // タイムスタンプが等しい場合は value が大きい方を返す（冪等性 + 決定論的マージ）
  return a.value >= b.value ? a : b;
}

// lwwEquals: 2 つの LWW レジスタが等価であるかを判定する関数
function lwwEquals(a: LWWRecord, b: LWWRecord): boolean {
  // value と timestamp の両方が等しければ等価とする
  return a.value === b.value && a.timestamp === b.timestamp;
}

// ---- テストケース ----

// runTests: 全プロパティテストを実行する
function runTests(): void {
  // テスト成功カウンタを初期化する
  let passed = 0;
  // テスト失敗カウンタを初期化する
  let failed = 0;

  // テスト実行ヘルパー関数
  function test(name: string, assertion: () => boolean): void {
    try {
      // assertion を実行する
      const result = assertion();
      // 結果を確認する
      if (result) {
        // 成功をカウントする
        passed++;
        // 成功メッセージを出力する
        console.log(`PASS: ${name}`);
      } else {
        // 失敗をカウントする
        failed++;
        // 失敗メッセージを出力する
        console.error(`FAIL: ${name}: assertion returned false`);
      }
    } catch (err) {
      // 例外をカウントする
      failed++;
      // 例外メッセージを出力する
      console.error(`FAIL: ${name}: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  // ---- テストデータ ----
  // state: 初期状態（timestamp=100, value=10）
  const state: LWWRecord = { value: 10, timestamp: 100 };
  // op1: 最初の操作（timestamp=200, value=20 — state より新しい）
  const op1: LWWRecord = { value: 20, timestamp: 200 };
  // op2: 2 番目の操作（timestamp=150, value=15 — op1 より古い）
  const op2: LWWRecord = { value: 15, timestamp: 150 };
  // op3: 3 番目の操作（timestamp=250, value=25 — op1 より新しい）
  const op3: LWWRecord = { value: 25, timestamp: 250 };

  // ---- 1. 交換律（Commutativity）の検証 ----
  // merge(merge(state, op1), op2) === merge(merge(state, op2), op1)
  test("LWW isCommutative (op1, op2)", () =>
    isCommutative(state, op1, op2, lwwMerge, lwwEquals)
  );

  // merge(merge(state, op2), op3) === merge(merge(state, op3), op2)
  test("LWW isCommutative (op2, op3)", () =>
    isCommutative(state, op2, op3, lwwMerge, lwwEquals)
  );

  // 同一 timestamp の場合の交換律（tie-breaking は value の大きい方が優先）
  const opA: LWWRecord = { value: 30, timestamp: 300 };
  const opB: LWWRecord = { value: 25, timestamp: 300 };
  test("LWW isCommutative (same timestamp, tie-breaking by value)", () =>
    isCommutative(state, opA, opB, lwwMerge, lwwEquals)
  );

  // ---- 2. 結合律（Associativity）の検証 ----
  // (state op1 op2) op3 === state op1 (state op2 op3) となる LWW の結合律
  test("LWW isAssociative (op1, op2, op3)", () =>
    isAssociative(state, op1, op2, op3, lwwMerge, lwwEquals)
  );

  // ---- 3. 冪等律（Idempotency）の検証 ----
  // merge(merge(state, op1), op1) === merge(state, op1)
  test("LWW isIdempotent (op1 applied twice)", () =>
    isIdempotent(state, op1, lwwMerge, lwwEquals)
  );

  // 自身とのマージが冪等であることを確認する
  test("LWW isIdempotent (state merged with itself)", () =>
    isIdempotent(state, state, lwwMerge, lwwEquals)
  );

  // ---- 4. 3 法則一括検証 ----
  test("LWW checkCRDTConvergence (all 3 laws)", () => {
    // 3 法則を一括検証する
    const result = checkCRDTConvergence(state, op1, op2, op3, lwwMerge, lwwEquals);
    // allPassed が true であることを確認する
    return result.allPassed;
  });

  // ---- 5. max-merge counter CRDT（単調増加カウンタ）の検証 ----
  // GCounter（Grow-only Counter）: max merge で交換律・結合律・冪等律を満たす
  // GCounter の状態は単純な number とし、merge は max で定義する
  const gcMerge = (a: number, b: number): number => Math.max(a, b);
  const gcEquals = (a: number, b: number): boolean => a === b;

  test("GCounter isCommutative (1, 2)", () =>
    isCommutative(0, 1, 2, gcMerge, gcEquals)
  );

  test("GCounter isAssociative (1, 2, 3)", () =>
    isAssociative(0, 1, 2, 3, gcMerge, gcEquals)
  );

  test("GCounter isIdempotent (5 applied twice)", () =>
    isIdempotent(0, 5, gcMerge, gcEquals)
  );

  test("GCounter checkCRDTConvergence (all 3 laws)", () => {
    // GCounter の 3 法則を一括検証する
    const result = checkCRDTConvergence(0, 3, 7, 5, gcMerge, gcEquals);
    // allPassed が true であることを確認する
    return result.allPassed;
  });

  // ---- 結果サマリ ----
  // 結果サマリを出力する
  console.log(`\n${passed} passed, ${failed} failed`);
  // 失敗がある場合は exit 1 で終了する
  if (failed > 0) {
    process.exit(1);
  }
}

// runTests を実行する
runTests();
